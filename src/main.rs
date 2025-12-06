use clap::Parser;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{self, execute};
use rand::seq::IndexedRandom;
use rodio::Source;
use rodio::source::SineWave;
use std::io::{Write, stdout};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, RwLock};
use std::time::Duration;

mod constants;

#[derive(clap::Parser)]
struct TrainArgs {
    /// Path to the words file
    #[clap(short, long)]
    words_file: Option<PathBuf>,
}

#[derive(clap::Subcommand, Clone)]
enum Mode {
    /// Practice writing morse code (default).
    Writing {
        /// Use telegraph keying paddles emulation
        #[clap(short, long, default_value_t = false)]
        paddle: bool,
        /// Don't show banner and controls info.
        #[clap(short, long, default_value_t = false)]
        minimal: bool,
        /// Silent mode (no sound)
        #[clap(short, long, default_value_t = false)]
        silent: bool,
    },
    /// Practice listening to morse code.
    Listening {
        /// Path to the words file.
        /// If not provided, you will be tested against single letters.
        #[clap(short, long)]
        words_file: Option<PathBuf>,
        /// User farnsworth timings.
        /// This mode helps recognize code better
        /// by giving more time for spaces between letters and words.
        /// Google farnsworth timing for more info.
        #[clap(long)]
        farnsworth: bool,
    },
}

#[derive(clap::Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Words per minute
    #[clap(short, long, default_value_t = 10)]
    wpm: u64,
    // Volume in percentage 1-100
    #[clap(short, long, default_value_t = 11)]
    volume: u8,
    /// Frequency in Hz
    #[clap(short, long, default_value_t = 1200)]
    frequency: u32,
    #[clap(subcommand)]
    mode: Option<Mode>,
}

/// IO events that we receive from users input
/// KeyPress and KeyRelease are sent before Dot/Dash events
/// to indicate the state of the key.
#[derive(Debug)]
enum IOEvent {
    KeyPress,
    KeyRelease,
    Dot,
    Dash,
    Clear,
    Exit,
}

// UI thread function
// Updates the terminal UI every `tic` duration
// with current state of the application.
//
// - tic: Duration for UI update interval
// - paddle: whether paddle mode is enabled
// - text_lock: shared lock for the translated text
// - buffer_lock: shared lock for the current Morse code buffer
// - ticks_lock: shared lock for the current tick count
fn ui(
    tic: Duration,
    paddle: bool,
    minimal: bool,
    text_lock: Arc<RwLock<String>>,
    buffer_lock: Arc<RwLock<Vec<char>>>,
    ticks_lock: Arc<RwLock<u64>>,
) -> anyhow::Result<()> {
    let mut contols = vec!["ESC or ^C => exit", "Backspace => clear text"];
    if paddle {
        contols.push("[ => Dot");
        contols.push("] => Dash");
    } else {
        contols.push("Any key => emit signal");
    }
    loop {
        std::thread::sleep(tic);
        execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        // We don't want to show banner in minimal mode.
        // Just ticks and text.
        if !minimal {
            println!("Morse Code Translator");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            println!("Controls:");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            for control in &contols {
                println!("\t{control}");
                execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            }
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            println!("Use backspace to clear text. Esc or ^C to exit.");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            println!("=====================");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
            print!("Ticks: ");
        }

        let ticks = ticks_lock.read().unwrap();

        for i in 0..7 {
            if i < *ticks || *ticks > 7 {
                print!("+");
            } else {
                print!("-");
            }
        }
        println!();
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        let text = text_lock.read().unwrap();
        let buffer = buffer_lock.read().unwrap();
        let chars = buffer.iter().collect::<String>();
        print!("{text}{chars}");
        stdout().flush()?;
    }
}

/// IO thread function for single key input mode.
///
/// Handles user input and plays sound accordingly.
///
/// sends events to the main thread via event_queue.
/// - tic: Duration for dot/dash timing
/// - silent: whether to play sound or not
/// - volume: volume of the sound (0-100)
/// - frequency: frequency of the sound in Hz
/// - event_queue: channel to send IOEvents to main thread
fn io_single_key(
    tic: Duration,
    silent: bool,
    volume: u8,
    frequency: u32,
    event_queue: std::sync::mpsc::Sender<IOEvent>,
) -> anyhow::Result<()> {
    // When last key was pressed
    let mut last_press = std::time::Instant::now();
    let mut holding = false;

    // Stream for output audio
    let mut stream = rodio::OutputStreamBuilder::open_default_stream().ok();
    stream.as_mut().map(|s| s.log_on_drop(false));
    let mut sink = stream.as_ref().map(|s| rodio::Sink::connect_new(s.mixer()));
    sink.as_ref().map(|s| s.set_volume(volume as f32 / 100.0));

    // If silent, we drop the sink to avoid playing sound
    if silent {
        sink.take();
    }

    loop {
        let event = crossterm::event::read()?;
        // Not a key event, ignore
        let Some(kev) = event.as_key_event() else {
            continue;
        };
        // Exit on ESC or Ctrl+C
        if kev.code.is_esc()
            || (kev.code.is_char('c')
                && kev
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL))
        {
            break;
        }
        // Clear event on Backspace
        if kev.code.is_backspace() {
            event_queue.send(IOEvent::Clear)?;
            continue;
        }

        // If this is a key press event
        // We just remember the time,
        // to later match it with release event.
        // Also we start playing sound on press
        // in a separate thread.
        if kev.is_press() {
            // To not play sound twice.
            if holding {
                continue;
            }
            holding = true;
            event_queue.send(IOEvent::KeyPress)?;
            sink.as_ref()
                .map(|s| s.append(SineWave::new(frequency as f32)));
            last_press = std::time::Instant::now();
        }
        // In case of release,
        // we want to compare the time elapsed
        // since last press to determine
        // whether it was a dot or dash.
        if kev.is_release() {
            sink.as_ref().map(|s| s.stop());
            holding = false;
            event_queue.send(IOEvent::KeyRelease)?;
            let now = std::time::Instant::now();
            let passed = now.duration_since(last_press);
            if passed <= tic {
                event_queue.send(IOEvent::Dot)?;
            } else {
                event_queue.send(IOEvent::Dash)?;
            }
        }
    }
    // At the end of the IO loop, we send Exit event.
    event_queue.send(IOEvent::Exit)?;
    Ok(())
}

/// An event used in paddle mode
/// to activate emitter.
///
/// When StartDot or StartDash is received,
/// the emitter thread will start emitting
/// corresponding signals continuously until
/// Stop is received.
///
/// On stop, it will stop any ongoing sound
/// and wait for new commands.
///
/// On exit, the emitter thread will terminate itself.
#[derive(Debug, Copy, Clone)]
enum PaddleEmitterEvent {
    StartDot,
    StartDash,
    Stop,
    Exit,
}

/// IO thread function for paddle mode.
///
/// Handles user input and plays sound accordingly.
///
/// sends events to the main thread via event_queue.
/// - tic: Duration for dot/dash timing
/// - silent: whether to play sound or not
/// - volume: volume of the sound (0-100)
/// - frequency: frequency of the sound in Hz
/// - event_queue: channel to send IOEvents to main thread
fn io_paddle(
    tic: Duration,
    silent: bool,
    volume: u8,
    frequency: u32,
    event_queue: std::sync::mpsc::Sender<IOEvent>,
) -> anyhow::Result<()> {
    // Stream for output audio

    // If silent, we drop the sink to avoid playing sound

    let (emitter_tx, emitter_rx) = std::sync::mpsc::channel::<PaddleEmitterEvent>();
    let event_tx_clone = event_queue.clone();

    // Continuous emitter thread.
    //
    // This thread dot and dash events. It's required, because
    // in paddle mode, user can press and hold either key,
    // and we need to send dot or dash events continuously.
    //
    // This is used to simulate real telegraph keying paddles behavior.
    std::thread::spawn(move || {
        // We open audio stream here. Because we don't need it outside.
        let mut stream = rodio::OutputStreamBuilder::open_default_stream().ok();
        stream.as_mut().map(|s| s.log_on_drop(false));
        let mut sink = stream.as_ref().map(|s| rodio::Sink::connect_new(s.mixer()));
        sink.as_ref().map(|s| s.set_volume(volume as f32 / 100.0));
        if silent {
            sink.take();
        }

        // Last command to repeat if no new command is received.
        let mut last_command = PaddleEmitterEvent::Stop;
        loop {
            let command = match emitter_rx.try_recv() {
                Ok(cmd) => cmd,
                Err(TryRecvError::Empty) => last_command,
                Err(TryRecvError::Disconnected) => PaddleEmitterEvent::Exit,
            };
            // For those look ad PaddleEmitterEvent docs.
            match command {
                PaddleEmitterEvent::StartDot => {
                    event_tx_clone.send(IOEvent::KeyPress).ok();
                    event_tx_clone.send(IOEvent::Dot).ok();
                    sink.as_ref()
                        .map(|s| s.append(SineWave::new(frequency as f32)));
                    std::thread::sleep(tic);
                    sink.as_ref().map(|s| s.stop());
                    event_tx_clone.send(IOEvent::KeyRelease).ok();
                }
                PaddleEmitterEvent::StartDash => {
                    event_tx_clone.send(IOEvent::KeyPress).ok();
                    event_tx_clone.send(IOEvent::Dash).ok();
                    sink.as_ref()
                        .map(|s| s.append(SineWave::new(frequency as f32)));
                    std::thread::sleep(3 * tic);
                    sink.as_ref().map(|s| s.stop());
                    event_tx_clone.send(IOEvent::KeyRelease).ok();
                }
                PaddleEmitterEvent::Stop => {
                    sink.as_ref().map(|s| s.stop());
                }
                PaddleEmitterEvent::Exit => break,
            }
            last_command = command;
        }
    });

    let mut pressed = false;

    loop {
        let event = crossterm::event::read()?;
        let Some(kev) = event.as_key_event() else {
            continue;
        };
        if kev.code.is_esc()
            || (kev.code.is_char('c')
                && kev
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL))
        {
            break;
        }
        if kev.code.is_backspace() {
            event_queue.send(IOEvent::Clear)?;
            continue;
        }

        if kev.is_press() {
            // We don't want to emit any futher events,
            // while the key is already pressed.
            if pressed {
                continue;
            }
            pressed = true;
            if kev.code == crossterm::event::KeyCode::Char('[') {
                emitter_tx.send(PaddleEmitterEvent::StartDot)?;
            } else if kev.code == crossterm::event::KeyCode::Char(']') {
                emitter_tx.send(PaddleEmitterEvent::StartDash)?;
            }
        }
        // When it's released, we just stop any ongoing sound.
        if kev.is_release() {
            pressed = false;
            emitter_tx.send(PaddleEmitterEvent::Stop)?;
        }
    }
    emitter_tx.send(PaddleEmitterEvent::Exit).ok();
    event_queue.send(IOEvent::Exit).ok();
    Ok(())
}

/// Main application function
fn writing_loop(
    wpm: u64,
    volume: u8,
    frequency: u32,
    silent: bool,
    paddle: bool,
    minimal: bool,
) -> anyhow::Result<String> {
    // We calculate one tic (which equals to one dot duration) for
    // target WPM using the following formula.
    // Word PARIS is used as standard word to calculate WPM
    // for MORSE code communications.
    //
    // PARIS = one word = 50 ticks (dots)
    //
    // Minutes per word = 1 / WPM
    // Seconds per word = 60 / WPM
    // Seconds per tick = (60 / WPM) / 50 = 60 / (50 * WPM)
    // Milliseconds per tick = (60 / (50 * WPM)) * 1000
    let tic = Duration::from_millis((60.0 / (50.0 * wpm as f64) * 1000.0) as u64);
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    // We use RwLock to share data between threads safely
    // and have less time blocked on locks when just reading.
    let data = Arc::new(RwLock::new(String::new()));
    let buffer = Arc::new(RwLock::new(Vec::new()));
    let ticks = Arc::new(RwLock::new(0u64));

    // Depending on the mode, we spawn different IO threads.
    if paddle {
        std::thread::spawn(move || io_paddle(tic.clone(), silent, volume, frequency, event_tx));
    } else {
        std::thread::spawn(move || io_single_key(tic.clone(), silent, volume, frequency, event_tx));
    }

    // We have to clone data before moving into the thread
    // because move closures take ownership of the variables.
    let ui_data = data.clone();
    let ui_buffer = buffer.clone();
    let ui_ticks = ticks.clone();
    std::thread::spawn(move || ui(tic, paddle, minimal, ui_data, ui_buffer, ui_ticks));

    let mut pressed = false;
    let mut last_tick = std::time::Instant::now();

    loop {
        // Handle for ticks.
        // We just constantly update the ticks counter based on time elapsed.
        {
            // We assume that this function will be BLAZINGLY fast to update
            // all the events. So therefore we can just check the elapsed time
            // since last tick and update accordingly.
            let elapsed = last_tick.elapsed();
            if elapsed > tic {
                let mut ticks_guard = ticks.write().unwrap();
                *ticks_guard = ticks_guard.saturating_add(1);
                last_tick = std::time::Instant::now();
                // If we have a key pressed, we do nothing more.
                // Because it just means a long press.
                // So we don't want to resolve buffer earlier
                // than user intends.
                if !pressed {
                    // Now check if we need to commit any data
                    // Once every 3 ticks we try resolving the buffer into a character
                    if *ticks_guard == 3 {
                        let mut buf = buffer.write().unwrap();
                        let buffer_chrs = buf.iter().collect::<String>();
                        for &(ch, code) in &constants::ABC {
                            // If we found a match, add it to a text.
                            if code == buffer_chrs {
                                let mut text = data.write().unwrap();
                                text.push(ch);
                                break;
                            }
                        }
                        // Remove the buffer either way
                        // So we can receive new input
                        buf.clear();
                    }
                    // If 7 ticks have passed without input, we commit a space
                    if *ticks_guard == 7 {
                        let mut text = data.write().unwrap();
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        // We can skip clearing buffer,
                        // because it should be already empty,
                        // but just in case. 🤷
                        let mut buf = buffer.write().unwrap();
                        buf.clear();
                    }
                }
            }
        }
        // Now we try receiving any IO events.
        let possible_event = event_rx.try_recv();

        match possible_event {
            // If there were no events, we just continue
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            // If the channel is disconnected by any reason, we exit the loop
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            // Otherwise we have an event to process
            Ok(io_event) => {
                // Here we update our last_tick,
                // because all IO events should reset
                // the tick counter.
                last_tick = std::time::Instant::now();
                match io_event {
                    IOEvent::Dot => {
                        let mut buf = buffer.write().unwrap();
                        let mut ticks_guard = ticks.write().unwrap();
                        buf.push('.');
                        *ticks_guard = 0;
                    }
                    IOEvent::Dash => {
                        let mut buf = buffer.write().unwrap();
                        let mut ticks_guard = ticks.write().unwrap();
                        buf.push('-');
                        *ticks_guard = 0;
                    }
                    IOEvent::Clear => {
                        let mut buf = buffer.write().unwrap();
                        let mut text = data.write().unwrap();
                        let mut ticks_guard = ticks.write().unwrap();
                        buf.clear();
                        text.clear();
                        *ticks_guard = 0;
                    }
                    IOEvent::KeyPress => {
                        pressed = true;
                        let mut ticks_guard = ticks.write().unwrap();
                        *ticks_guard = 0;
                    }
                    IOEvent::KeyRelease => {
                        pressed = false;
                        let mut ticks_guard = ticks.write().unwrap();
                        *ticks_guard = 0;
                    }
                    IOEvent::Exit => break,
                }
            }
        }
    }
    if data.is_poisoned() {
        data.clear_poison();
    }
    let text = if let Ok(data_lock) = data.read() {
        data_lock.clone()
    } else {
        String::new()
    };

    Ok(text)
}

fn start_writing(args: Args, silent: bool, paddle: bool, minimal: bool) -> anyhow::Result<()> {
    enable_raw_mode()?;

    #[cfg(not(windows))]
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    // We will handle writing mode below
    let result = writing_loop(
        args.wpm,
        args.volume,
        args.frequency,
        silent,
        paddle,
        minimal,
    );
    execute!(
        stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    stdout().flush()?;

    #[cfg(not(windows))]
    execute!(stdout(), PopKeyboardEnhancementFlags)?;

    disable_raw_mode()?;

    match result {
        Ok(text) => {
            print!("{text}");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    stdout().flush().ok();
    Ok(())
}

fn enqueue_word(
    sink: &rodio::Sink,
    farnsworth: bool,
    word: &str,
    wpm: u64,
    frequency: u32,
) -> anyhow::Result<()> {
    let dit = Duration::from_millis((60.0 / (50.0 * wpm as f64) * 1000.0) as u64);
    let dah = 3 * dit;

    let mut between_letters = 3 * dit;
    let mut between_words = 7 * dit;

    if farnsworth {
        between_letters = between_letters * 2;
        between_words = between_words * 2;
    }

    for (i, ch) in word.chars().enumerate() {
        for &(c, code) in &constants::ABC {
            if c == ch {
                for (j, signal) in code.chars().enumerate() {
                    let duration = match signal {
                        '.' => dit,
                        '-' => dah,
                        _ => continue,
                    };
                    sink.append(
                        rodio::source::SineWave::new(frequency as f32).take_duration(duration),
                    );
                    if j < code.len() - 1 {
                        // Space between signals in a letter
                        sink.append(rodio::source::Zero::new(1, 48000).take_duration(dit));
                    }
                }
                break;
            }
        }
        if ch != ' ' && i < word.len() - 1 {
            // Space between letters
            sink.append(rodio::source::Zero::new(1, 48000).take_duration(between_letters));
        }
        if ch == ' ' {
            // Space between words (7 dits)
            sink.append(rodio::source::Zero::new(1, 48000).take_duration(between_words));
        }
    }
    Ok(())
}

fn listening_loop(args: Args, farnsworth: bool, words_file: Option<PathBuf>) -> anyhow::Result<()> {
    println!("Morse code listening training.");
    println!(
        "WPM: {}, Frequency: {} Hz, Volume: {}%",
        args.wpm, args.frequency, args.volume
    );
    println!(
        "You are going to hear Morse code. Translate it to text, write in the console and press enter to submit."
    );
    println!("Leave empty and press enter to hear the code again.");
    println!("Press Ctrl+C to exit.");
    println!("============================");
    stdout().flush()?;
    let stream = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream.mixer());
    sink.set_volume(args.volume as f32 / 100.0);
    let mut training_words = vec![];

    if let Some(path) = words_file {
        println!("Using words file: {:?}", path);
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            let word = line.trim();
            if !word.is_empty() {
                training_words.push(word.trim().to_uppercase());
            }
        }
    } else {
        println!("No words file provided. Using single letters for training.");
        // Here we would implement the listening training logic
        // using single letters.
        for &(ch, _) in &constants::ABC {
            training_words.push(ch.to_string());
        }
    }
    let mut rng = rand::rng();

    loop {
        let Some(target) = training_words.choose(&mut rng) else {
            anyhow::bail!("No training words available.");
        };
        let mut answer = String::new();
        while answer.trim().is_empty() {
            sink.stop();
            enqueue_word(
                &sink,
                farnsworth,
                &target.to_uppercase(),
                args.wpm,
                args.frequency,
            )?;
            print!(">");
            stdout().flush()?;
            sink.play();
            std::io::stdin().read_line(&mut answer)?;
        }
        if answer.trim().to_uppercase() == *target {
            println!("Correct!");
        } else {
            println!("Incorrect! The correct answer was: {}", target);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.clone().mode {
        Some(Mode::Listening {
            words_file,
            farnsworth,
        }) => {
            listening_loop(args, farnsworth, words_file)?;
            return Ok(());
        }
        Some(Mode::Writing {
            paddle,
            minimal,
            silent,
        }) => {
            start_writing(args, silent, paddle, minimal)?;
        }
        None => {
            // Default to writing mode without paddles and minimal UI
            start_writing(args, false, false, false)?;
        }
    }

    Ok(())
}
