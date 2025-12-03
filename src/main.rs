use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll, read,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{self, execute};
use std::io::{Write, stdout};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    #[cfg(not(windows))]
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let abc = [
        ('A', ".-"),
        ('B', "-..."),
        ('C', "-.-."),
        ('D', "-.."),
        ('E', "."),
        ('F', "..-."),
        ('G', "--."),
        ('H', "...."),
        ('I', ".."),
        ('J', ".---"),
        ('K', "-.-"),
        ('L', ".-.."),
        ('M', "--"),
        ('N', "-."),
        ('O', "---"),
        ('P', ".--."),
        ('Q', "--.-"),
        ('R', ".-."),
        ('S', "..."),
        ('T', "-"),
        ('U', "..-"),
        ('V', "...-"),
        ('W', ".--"),
        ('X', "-..-"),
        ('Y', "-.--"),
        ('Z', "--.."),
        ('1', ".----"),
        ('2', "..---"),
        ('3', "...--"),
        ('4', "....-"),
        ('5', "....."),
        ('6', "_...."),
        ('7', "--..."),
        ('8', "---.."),
        ('9', "----."),
        ('0', "-----"),
        ('.', ".-.-.-"),
    ];
    let wpm = 5.0;

    let tic = (60.0 / (50.0 * wpm) * 1000.0) as u64;

    let mut last_press = std::time::Instant::now();

    let mut text = String::new();
    let mut buffer = Vec::<bool>::new();
    let mut empties = 0;
    let mut holding = false;

    loop {
        if poll(Duration::from_millis(tic))? {
            empties = 0;
            let event = read()?;
            match event {
                crossterm::event::Event::Key(kev) => {
                    if kev.code.is_esc()
                        || (kev.code.is_char('c')
                            && kev
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL))
                    {
                        print!("\nExiting! Bye!\n");
                        stdout().flush()?;
                        break;
                    }
                    if kev.code.is_backspace() {
                        buffer.clear();
                        text.clear();
                        execute!(
                            stdout(),
                            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                        )?;
                    }
                    if kev.is_press() {
                        holding = true;
                        last_press = std::time::Instant::now();
                    }
                    if kev.is_release() {
                        holding = false;
                        let now = std::time::Instant::now();
                        let passed = now.duration_since(last_press);
                        let mut is_dit = false;
                        if passed.as_millis() < tic as u128 {
                            is_dit = true;
                        }
                        buffer.push(is_dit);
                    }
                }
                _ => {}
            }
        } else {
            if holding {
                continue;
            }
            empties += 1;
            if empties == 1 {
                for &(ch, code) in &abc {
                    if code.len() == buffer.len() {
                        let mut matched = true;
                        for (i, c) in code.chars().enumerate() {
                            let is_dit = c == '.';
                            if buffer[i] != is_dit {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            text.push(ch);
                            print!("{}", ch);
                            stdout().flush()?;
                            break;
                        }
                    }
                }
                buffer.clear();
            }
            if empties == 7 {
                buffer.clear();
                text.push(' ');
                print!(" ");
                stdout().flush()?;
            }
        }
    }

    #[cfg(not(windows))]
    execute!(stdout(), PopKeyboardEnhancementFlags)?;

    disable_raw_mode()?;
    Ok(())
}
