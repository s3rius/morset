use clap::Parser;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll, read,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{self, execute};
use rodio::source::SineWave;
use std::io::{Write, stderr, stdout};
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut stream = rodio::OutputStreamBuilder::open_default_stream().ok();
    stream.as_mut().map(|s| s.log_on_drop(false));
    let mut sink = stream.as_ref().map(|s| rodio::Sink::connect_new(s.mixer()));
    if args.silent {
        sink.take();
    }
    sink.as_ref()
        .map(|s| s.set_volume(args.volume as f32 / 100.0));

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
    let wpm = args.wpm as f64;

    let tic = (60.0 / (50.0 * wpm) * 1000.0) as u64;

    let mut last_press = std::time::Instant::now();

    let mut text = String::new();
    let mut buffer = Vec::<bool>::new();
    let mut empties = 0;
    let mut holds = 0;
    let mut holding = false;

    loop {
        execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        if holding {
            for i in 0..7 {
                if i < holds || holds > 7 {
                    print!("●");
                } else {
                    print!(" ");
                }
            }
        } else {
            for i in 0..7 {
                if i < empties || empties > 7 {
                    print!("●");
                } else {
                    print!(" ");
                }
            }
        }
        println!();
        let chars = buffer
            .iter()
            .map(|&b| if b { '.' } else { '-' })
            .collect::<String>();
        execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        print!("{text}{chars}");
        stdout().flush()?;
        if poll(Duration::from_millis(tic))? {
            empties = 0;
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
                stdout().flush()?;
                break;
            }
            if kev.code.is_backspace() {
                buffer.clear();
                text.clear();
                continue;
            }

            if args.paddle {
                if kev.is_press() {
                    match kev.code {
                        crossterm::event::KeyCode::Char('[') => {
                            sink.as_ref()
                                .map(|s| s.append(SineWave::new(args.frequency as f32)));
                            std::thread::sleep(Duration::from_millis(tic));
                            sink.as_ref().map(|s| s.stop());
                            buffer.push(true);
                        }
                        crossterm::event::KeyCode::Char(']') => {
                            sink.as_ref()
                                .map(|s| s.append(SineWave::new(args.frequency as f32)));
                            std::thread::sleep(Duration::from_millis(tic * 3));
                            sink.as_ref().map(|s| s.stop());
                            buffer.push(false);
                        }
                        _ => {}
                    }
                }
            } else {
                if kev.is_press() {
                    if holding {
                        holds += 1;
                        continue;
                    }
                    holding = true;
                    sink.as_ref()
                        .map(|s| s.append(SineWave::new(args.frequency as f32)));
                    last_press = std::time::Instant::now();
                }
                if kev.is_release() {
                    sink.as_ref().map(|s| s.stop());
                    holding = false;
                    holds = 0;
                    let now = std::time::Instant::now();
                    let passed = now.duration_since(last_press);
                    let mut is_dit = false;
                    if passed.as_millis() < tic as u128 {
                        is_dit = true;
                    }
                    buffer.push(is_dit);
                }
            }
        } else {
            if holding {
                continue;
            }
            empties += 1;
            if empties == 3 {
                let buffer_chrs = buffer
                    .iter()
                    .map(|&b| if b { '.' } else { '-' })
                    .collect::<String>();

                for &(ch, code) in &constants::ABC {
                    if code.len() == buffer_chrs.len() {
                        if code == buffer_chrs {
                            text.push(ch);
                            break;
                        }
                    }
                }
                buffer.clear();
            }
            if empties == 7 {
                if text.is_empty() {
                    continue;
                }
                buffer.clear();
                text.push(' ');
                stdout().flush()?;
            }
        }
    }

    sink.as_ref().map(|s| s.stop());

    #[cfg(not(windows))]
    execute!(stdout(), PopKeyboardEnhancementFlags)?;

    disable_raw_mode()?;
    Ok(())
}
