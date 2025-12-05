use clap::Parser;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, read,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{self, execute};
use rodio::source::SineWave;
use std::io::{Write, stdout};
use std::sync::{Arc, RwLock};
use std::time::Duration;

mod constants;

#[derive(clap::Parser)]
struct Args {
    /// Words per minute
    #[clap(short, long, default_value_t = 10)]
    wpm: u64,
    /// Silent mode (no sound)
    #[clap(short, long, default_value_t = false)]
    silent: bool,
    // Volume in percentage 1-100
    #[clap(short, long, default_value_t = 11)]
    volume: u8,
    /// Frequency in Hz
    #[clap(short, long, default_value_t = 1200)]
    frequency: u32,
    /// Use telegraph keying paddles emulation
    #[clap(short, long, default_value_t = false)]
    paddle: bool,
}

#[derive(Debug)]
enum IOEvent {
    KeyPress,
    KeyRelease,
    Dot,
    Dash,
    Clear,
    Exit,
}

fn ui(
    tic: Duration,
    paddle: bool,
    text_lock: Arc<RwLock<String>>,
    buffer_lock: Arc<RwLock<Vec<char>>>,
    ticks_lock: Arc<RwLock<u64>>,
) -> anyhow::Result<()> {
    loop {
        std::thread::sleep(tic);
        execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        println!("Morse Code Translator");
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        if paddle {
            println!("Press `[` for dot and `]` for dash");
        } else {
            println!("Press and hold any key to send Morse code");
        }
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        println!("=====================");
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        print!("Ticks: ");

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
        println!("Output: ");
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        let text = text_lock.read().unwrap();
        let buffer = buffer_lock.read().unwrap();
        let chars = buffer.iter().collect::<String>();
        print!("{text}{chars}");
        stdout().flush()?;
    }
}

fn io(
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
        let event = read()?;
        let Some(kev) = event.as_key_event() else {
            continue;
        };
        if kev.code.is_esc()
            || (kev.code.is_char('c')
                && kev
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL))
        {
            event_queue.send(IOEvent::Exit)?;
            break;
        }
        if kev.code.is_backspace() {
            event_queue.send(IOEvent::Clear)?;
            continue;
        }

        if kev.is_press() {
            if holding {
                continue;
            }
            holding = true;
            event_queue.send(IOEvent::KeyPress)?;
            sink.as_ref()
                .map(|s| s.append(SineWave::new(frequency as f32)));
            last_press = std::time::Instant::now();
        }
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
    Ok(())
}

fn run_app(args: Args) -> anyhow::Result<String> {
    let wpm = args.wpm as f64;
    let tic = Duration::from_millis((60.0 / (50.0 * wpm) * 1000.0) as u64);
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let data = Arc::new(RwLock::new(String::new()));
    let buffer = Arc::new(RwLock::new(Vec::new()));
    let ticks = Arc::new(RwLock::new(0u64));

    std::thread::spawn(move || {
        io(
            tic.clone(),
            args.silent,
            args.volume,
            args.frequency,
            event_tx,
        )
    });
    let ui_data = data.clone();
    let ui_buffer = buffer.clone();
    let ui_ticks = ticks.clone();
    std::thread::spawn(move || ui(tic, args.paddle, ui_data, ui_buffer, ui_ticks));

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
                        let mut buf = buffer.write().unwrap();
                        buf.clear();
                    }
                }
            }
        }
        // Now we try receiving any IO events.
        let possible_event = event_rx.try_recv();

        match possible_event {
            Ok(io_event) => {
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
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Welcome to Morse Code Translator!");
    if args.paddle {
        println!("Press `[` for dot and `]` for dash. Esc or ^C to exit.");
    } else {
        println!("Press and hold any key to send Morse code. Esc or ^C to exit.");
    }

    enable_raw_mode()?;

    #[cfg(not(windows))]
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;

    let result = run_app(args);

    // loop {
    //     execute!(
    //         stdout(),
    //         crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
    //         crossterm::cursor::MoveTo(0, 0)
    //     )?;
    //     if holding {
    //     } else {
    //         for i in 0..7 {
    //             if i < empties || empties > 7 {
    //                 print!("●");
    //             } else {
    //                 print!(" ");
    //             }
    //         }
    //     }
    //     println!();
    //     let chars = buffer
    //         .iter()
    //         .map(|&b| if b { '.' } else { '-' })
    //         .collect::<String>();
    //     execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
    //     print!("{text}{chars}");
    //     stdout().flush()?;
    //     if poll(Duration::from_millis(tic))? {
    //         empties = 0;
    //         let event = read()?;
    //     } else {
    //         if holding {
    //             continue;
    //         }
    //         empties += 1;
    //         if empties == 3 {
    //             let buffer_chrs = buffer
    //                 .iter()
    //                 .map(|&b| if b { '.' } else { '-' })
    //                 .collect::<String>();
    //
    //             for &(ch, code) in &constants::ABC {
    //                 if code == buffer_chrs {
    //                     text.push(ch);
    //                     break;
    //                 }
    //             }
    //             buffer.clear();
    //         }
    //         if empties == 7 {
    //             if text.is_empty() {
    //                 continue;
    //             }
    //             buffer.clear();
    //             text.push(' ');
    //             stdout().flush()?;
    //         }
    //     }
    // }

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
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    Ok(())
}
