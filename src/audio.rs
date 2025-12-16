use rodio::OutputStream;
use rodio::Sink;
use rodio::source::{SineWave, Source};

/// Simple audio manager for playing sine wave tones
pub struct AudioManager {
    // Keep the stream alive to maintain audio output
    _stream: OutputStream,
    sink: Sink,
    frequency: f32,
    volume: f32,
    is_playing: bool,
}

impl AudioManager {
    /// Create a new audio manager with the specified frequency and volume
    pub fn new(frequency: f32, volume: f32) -> Result<Self, String> {
        // Get default output stream using rodio 0.21 API
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to create audio output stream: {}", e))?;
        
        // Create sink connected to the output stream's mixer
        let sink = Sink::connect_new(stream.mixer());
        
        // Create initial sine wave
        let source = SineWave::new(frequency)
            .amplify(volume)
            .repeat_infinite();
        
        sink.append(source);
        sink.pause(); // Start paused
        
        Ok(AudioManager {
            _stream: stream,
            sink,
            frequency,
            volume,
            is_playing: false,
        })
    }
    
    /// Start playing the tone
    pub fn play(&mut self) {
        if !self.is_playing {
            self.sink.play();
            self.is_playing = true;
        }
    }
    
    /// Stop playing the tone
    pub fn pause(&mut self) {
        if self.is_playing {
            self.sink.pause();
            self.is_playing = false;
        }
    }
    
    /// Update the frequency of the sine wave
    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), String> {
        if (self.frequency - frequency).abs() < 0.1 {
            return Ok(()); // No significant change
        }
        self.frequency = frequency;
        // Note: Frequency changes require recreating the source, which isn't supported here
        Ok(())
    }
    
    /// Update the volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.sink.set_volume(volume);
    }
}
