use egui::{self, Key, RichText};
use std::time::Duration;

use crate::{
    audio::AudioManager,
    inputs::InputStateExt,
    state::AppState,
    utils::{morse_to_char, wpm_to_dit_duration},
};

pub static MAX_WPM: u8 = 40;
pub static MIN_WPM: u8 = 1;

pub static MAX_FREQUENCY: usize = 1200;
pub static MIN_FREQUENCY: usize = 300;

pub static MAX_VOLUME: usize = 100;
pub static MIN_VOLUME: usize = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyerMode {
    IambicA,
    IambicB,
    Straight,
}

pub enum IambicKey {
    Dot,
    Dash,
}

#[derive(Default)]
pub struct IambicScheduler {
    dot_presed: bool,
    dash_pressed: bool,
    queue: Vec<IambicKey>,
    stop_at: Option<usize>,
}

impl IambicScheduler {
    pub fn press_key(&mut self, key: IambicKey) {
        unimplemented!()
    }
    pub fn release_key(&mut self, key: IambicKey) {
        unimplemented!()
    }
    pub fn emit_signal(&self, tick: usize) -> bool {
        unimplemented!()
    }

    pub fn any_active(&self) -> bool {
        self.dot_presed || self.dash_pressed
    }
}

/// Main structure that
/// tracks the timing of Morse code elements.
#[derive(Default, Debug)]
pub struct Ticker {
    pub ticks: usize,
    pub dit_duration: Duration,
    elapsed: Duration,
    was_reset: bool,
}

impl Ticker {
    pub fn new(dit_duration: Duration) -> Self {
        Self {
            dit_duration,
            ..Default::default()
        }
    }

    /// Reset the ticker to zero ticks.
    pub fn reset(&mut self) {
        tracing::debug!("Ticker reset scheduled");
        self.was_reset = true;
        self.ticks = 0;
        self.elapsed = Duration::ZERO;
    }

    /// Advance the ticker by delta time.
    ///
    /// Returns Some(new_ticks) if the tick count has changed,
    /// or None if it remains the same.
    pub fn tick(&mut self, delta: Duration) -> Option<usize> {
        let was_reset = self.was_reset;
        self.was_reset = false;
        self.elapsed += delta;

        let old_ticks = self.ticks;
        while self.elapsed >= self.dit_duration {
            self.elapsed -= self.dit_duration;
            if self.ticks < 7 {
                self.ticks += 1;
            }
        }

        // If it was reset, then we guarantee a tick update.
        // Because it's a new cycle.
        if was_reset || old_ticks != self.ticks {
            Some(self.ticks)
        } else {
            None
        }
    }
}

pub struct WritingScreen {
    // Display state
    text: String,
    buffer: Vec<char>,

    keyer_mode: KeyerMode,

    // Private state
    ticker: Ticker,
    pressed: bool,
    cheat_sheet_open: bool,

    /// User settings
    frequency: usize,
    volume: usize,
    wpm: u8,
}

impl WritingScreen {
    pub fn new() -> Self {
        let wpm = 10;
        let dit_duration = wpm_to_dit_duration(wpm);

        Self {
            text: String::new(),
            buffer: Vec::new(),
            ticker: Ticker::new(dit_duration),
            wpm,
            keyer_mode: KeyerMode::Straight,
            frequency: 550,
            pressed: false,
            cheat_sheet_open: false,
            volume: 20,
        }
    }

    /// This function just verifies that all values are within bounds.
    fn normalize_values(&mut self) {
        self.wpm = self.wpm.clamp(MIN_WPM, MAX_WPM);
        self.frequency = self.frequency.clamp(MIN_FREQUENCY, MAX_FREQUENCY);
        self.volume = self.volume.clamp(MIN_VOLUME, MAX_VOLUME);
        let dit_duration = wpm_to_dit_duration(self.wpm);
        if self.ticker.dit_duration != dit_duration {
            self.ticker.dit_duration = dit_duration;
            self.ticker.reset();
        }
    }

    /// Update the screen and return new state if changed
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        delta: Duration,
        audio: &mut Option<AudioManager>,
    ) -> Option<AppState> {
        let mut new_state = None;

        // Handle timing
        self.handle_timers(delta);

        // Handle input
        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                new_state = Some(AppState::MainMenu);
            } else if i.key_pressed(Key::Backspace) {
                self.text.clear();
                self.buffer.clear();
            } else if i.key_pressed(Key::F1) {
                self.wpm = self.wpm.saturating_sub(1);
                self.normalize_values();
            } else if i.key_pressed(Key::F2) {
                self.wpm = self.wpm.saturating_add(1);
                self.normalize_values();
            } else if i.key_pressed(Key::F3) {
                self.frequency = self.frequency.saturating_sub(50);
                if let Some(audio) = audio {
                    audio.set_frequency(self.frequency as f32);
                }
            } else if i.key_pressed(Key::F4) {
                self.frequency = self.frequency.saturating_add(50);
                if let Some(audio) = audio {
                    audio.set_frequency(self.frequency as f32);
                }
            } else if i.key_pressed(Key::F5) {
                self.volume = self.volume.saturating_sub(5);
                if let Some(audio) = audio {
                    audio.set_volume(self.volume as f32 * 0.01);
                }
            } else if i.key_pressed(Key::F6) {
                self.volume = self.volume.saturating_add(5);
                if let Some(audio) = audio {
                    audio.set_volume(self.volume as f32 * 0.01);
                }
            } else if i.key_pressed(Key::C) {
                self.cheat_sheet_open = !self.cheat_sheet_open;
            }
            // Handle space key for morse code
            else if self.keyer_mode == KeyerMode::Straight && i.key_just_pressed(Key::Space) {
                tracing::debug!("Start emitting wave");
                self.pressed = true;
                self.ticker.reset();
                if let Some(audio) = audio {
                    audio.play();
                }
            } else if self.keyer_mode == KeyerMode::Straight && i.key_released(Key::Space) {
                tracing::debug!("Stop emitting wave");
                self.pressed = false;
                if let Some(audio) = audio {
                    audio.pause();
                }
                // Add dot or dash based on how long it was pressed
                if self.ticker.ticks <= 2 {
                    self.buffer.push('.');
                } else {
                    self.buffer.push('-');
                }
                self.ticker.reset();
            }
        });

        // Render UI
        self.render_ui(ctx, audio);

        new_state
    }

    fn handle_timers(&mut self, delta: Duration) {
        let Some(tick) = self.ticker.tick(delta) else {
            return;
        };
        tracing::debug!("Tick advanced to {}", tick);

        // If the key is being pressed, do not do anything.
        if self.pressed {
            return;
        }

        if tick == 3 {
            if let Some(ch) = morse_to_char(&self.buffer.iter().collect::<String>()) {
                self.text.push(ch);
            }
            for (prosign, seq) in crate::consts::PROSIGNS.iter() {
                if &self.buffer.iter().collect::<String>() == seq {
                    self.text.push_str(&prosign.to_string());
                }
            }
            // No matter if we found a value or not,
            // we need to clear up the buffer anyways.
            self.buffer.clear();
        } else if tick == 7 && !self.text.is_empty() && !self.text.ends_with(' ') {
            self.text.push(' ');
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context, audio: &mut Option<AudioManager>) {
        // Top panel with ticks
        egui::TopBottomPanel::top("Ticks").show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let ticks_info = (1..=7)
                    .map(|i| if i <= self.ticker.ticks { '+' } else { '-' })
                    .collect::<String>();
                ui.label(RichText::new(ticks_info).size(25.));
            });
        });

        // Bottom panel with controls
        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Controls:");
                ui.horizontal(|ui| {
                    let mut controls = [
                        ("Esc", "Return to Main Menu"),
                        ("Bksp", "Clear text"),
                        ("F1", "Decrease WPM"),
                        ("F2", "Increase WPM"),
                        ("F3", "Decrease frequency"),
                        ("F4", "Increase frequency"),
                        ("F5", "Decrease volume"),
                        ("F6", "Increase volume"),
                        ("C", "Toggle cheat sheet"),
                    ]
                    .to_vec();
                    match self.keyer_mode {
                        KeyerMode::IambicA | KeyerMode::IambicB => {
                            controls.extend_from_slice(&[("[", "Send dit"), ("]", "Send dash")]);
                        }
                        KeyerMode::Straight => {
                            controls.push(("Space", "Send Morse Code"));
                        }
                    }
                    ui.vertical(|ui| {
                        for (key, value) in controls {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:<8} - {}", key, value));
                            });
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Settings:");
                        ui.horizontal(|ui| {
                            ui.label("WPM:");
                            let wpm = ui.add(egui::Slider::new(&mut self.wpm, MIN_WPM..=MAX_WPM));
                            if wpm.changed() {
                                self.normalize_values();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Frequency:");
                            let frequency = ui.add(egui::Slider::new(
                                &mut self.frequency,
                                MIN_FREQUENCY..=MAX_FREQUENCY,
                            ));
                            if let Some(audio) = audio {
                                if frequency.changed() {
                                    audio.set_frequency(self.frequency as f32);
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Volume:");
                            let volume = ui
                                .add(egui::Slider::new(&mut self.volume, MIN_VOLUME..=MAX_VOLUME));

                            if let Some(audio) = audio {
                                if volume.changed() {
                                    audio.set_volume(self.volume as f32 * 0.01);
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Cheat sheet:");
                            ui.checkbox(&mut self.cheat_sheet_open, "");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Keyer Mode:");
                            egui::ComboBox::from_id_salt("keyer_mode")
                                .selected_text(format!("{:?}", self.keyer_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.keyer_mode,
                                        KeyerMode::Straight,
                                        "Straight",
                                    );
                                    ui.selectable_value(
                                        &mut self.keyer_mode,
                                        KeyerMode::IambicA,
                                        "Iambic A",
                                    );
                                    ui.selectable_value(
                                        &mut self.keyer_mode,
                                        KeyerMode::IambicB,
                                        "Iambic B",
                                    );
                                });
                        })
                    });
                });
            });
        });

        // Main text area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let buff = self.buffer.iter().collect::<String>();
                ui.label(egui::RichText::new(format!("{}{}|", self.text, buff)).size(32.));
            });
        });

        // Cheat sheet window
        egui::Window::new("Cheatsheet")
            .collapsible(true)
            .open(&mut self.cheat_sheet_open)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let codes = crate::consts::ABC
                        .iter()
                        .chain(crate::consts::NUMBERS.iter())
                        .chain(crate::consts::SIGNS.iter())
                        .collect::<Vec<_>>();
                    let middle = codes.len() / 2;

                    ui.vertical(|ui| {
                        for (id, (ch, seq)) in codes.iter().enumerate() {
                            if id <= middle {
                                ui.label(
                                    RichText::new(format!("{}: {}", ch, seq))
                                        .monospace()
                                        .size(20.),
                                );
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        for (id, (ch, seq)) in codes.iter().enumerate() {
                            if id > middle {
                                ui.label(
                                    RichText::new(format!("{}: {}", ch, seq))
                                        .monospace()
                                        .size(20.),
                                );
                            }
                        }
                    });
                });
            });
    }
}
