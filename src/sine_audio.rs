use std::time::Duration;

use bevy::{audio::Source, prelude::*};

#[derive(Asset, TypePath, Debug)]
pub struct SineAudio {
    pub frequency: f32,
}

pub struct SineDecoder {
    // how far along one period the wave is (between 0 and 1)
    current_progress: f32,
    // how much we move along the period every frame
    progress_per_frame: f32,
    // how long a period is
    period: f32,
    sample_rate: u32,
}

impl SineDecoder {
    fn new(frequency: f32) -> Self {
        // standard sample rate for most recordings
        let sample_rate = 44_100;
        SineDecoder {
            current_progress: 0.,
            progress_per_frame: frequency / sample_rate as f32,
            period: std::f32::consts::PI * 2.,
            sample_rate,
        }
    }
}

impl Iterator for SineDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_progress += self.progress_per_frame;
        // we loop back round to 0 to avoid floating point inaccuracies
        self.current_progress %= 1.;
        Some(ops::sin(self.period * self.current_progress))
    }
}

impl Source for SineDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for SineAudio {
    type DecoderItem = <SineDecoder as Iterator>::Item;

    type Decoder = SineDecoder;

    fn decoder(&self) -> Self::Decoder {
        SineDecoder::new(self.frequency)
    }
}
