use crate::consts;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use web_time::Duration;

pub fn wpm_to_dit_duration(wpm: u8) -> std::time::Duration {
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
    Duration::from_millis((1.2 * (1000. / wpm as f64)).ceil() as u64)
}

pub fn morse_to_char(morse: &str) -> Option<char> {
    for (c, code) in consts::ABC
        .iter()
        .chain(consts::NUMBERS.iter())
        .chain(consts::SIGNS.iter())
    {
        if *code == morse {
            return Some(*c);
        }
    }
    None
}
