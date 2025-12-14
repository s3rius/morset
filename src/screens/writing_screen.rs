use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, RichText},
};

use crate::{
    sine_audio::SineAudio,
    state::AppState,
    utils::{morse_to_char, wpm_to_dit_duration},
};

pub static MAX_WPM: u8 = 40;
pub static MIN_WPM: u8 = 1;

pub static MAX_FREQUENCY: usize = 1200;
pub static MIN_FREQUENCY: usize = 100;

pub static MAX_VOLUME: usize = 100;
pub static MIN_VOLUME: usize = 0;

pub struct WritingScreenPlugin;

impl Plugin for WritingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Writing), startup)
            .add_message::<SoundEvent>()
            .add_systems(
                EguiPrimaryContextPass,
                egui_menus.run_if(in_state(AppState::Writing)),
            )
            .add_systems(
                Update,
                (
                    controls,
                    timers,
                    sync_audio_values,
                    sound_system.after(controls),
                )
                    .distributive_run_if(in_state(AppState::Writing)),
            )
            .add_systems(OnExit(AppState::Writing), shutdown);
    }
}

#[derive(Clone, Resource)]
pub struct WritingState {
    // Display state
    text: String,
    buffer: Vec<char>,

    // Private state
    timer: Timer,
    ticks: usize,
    pressed: bool,

    /// User settings
    frequency: usize,
    volume: usize,
    wpm: u8,
}

#[derive(Message)]
pub enum SoundEvent {
    Start,
    Stop,
}

impl WritingState {
    /// Reset the timer and tick count.
    pub fn reset_timer(&mut self) {
        self.timer.reset();
        self.ticks = 0;
    }

    /// Progress the timer and see if value has been updated.
    pub fn tick(&mut self, delta: Duration) -> Option<usize> {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            if self.ticks < 7 {
                self.ticks += 1;
            }
            return Some(self.ticks);
        }
        return None;
    }

    /// When updating WPM, we need to update
    /// our timer as well.
    pub fn set_wpm(&mut self, wpm: u8) {
        self.wpm = wpm;
        self.ticks = 0;
        self.timer = Timer::new(wpm_to_dit_duration(self.wpm), TimerMode::Repeating);
    }

    /// This function just verifies that all values are within bounds.
    /// and updates the timer if WPM has changed.
    pub fn normalize_values(&mut self) {
        self.wpm = self.wpm.clamp(MIN_WPM, MAX_WPM);
        self.frequency = self.frequency.clamp(MIN_FREQUENCY, MAX_FREQUENCY);
        self.volume = self.volume.clamp(MIN_VOLUME, MAX_VOLUME);
        let dit_duration = wpm_to_dit_duration(self.wpm);
        if self.timer.duration() != dit_duration {
            self.timer = Timer::new(dit_duration, TimerMode::Repeating);
            self.reset_timer()
        }
    }
}

/// Initial system.
/// Here we just initialize the state and
pub fn startup(mut cmds: Commands) {
    let mut state = WritingState {
        text: String::new(),
        buffer: Vec::new(),
        timer: Timer::from_seconds(0.0, TimerMode::Repeating),
        ticks: 0,
        wpm: 10,
        frequency: 550,
        pressed: false,
        volume: 20,
    };
    state.set_wpm(10);
    cmds.insert_resource(state);
}

pub fn shutdown(mut cmds: Commands, audio_sink: Query<(&mut AudioSink, Entity)>) {
    // Remove state
    cmds.remove_resource::<WritingState>();
    // Despawn audio entities
    for (_, entity) in audio_sink {
        cmds.entity(entity).despawn();
    }
}

/// Main UI
pub fn egui_menus(mut contexts: EguiContexts, mut state: ResMut<WritingState>) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::TopBottomPanel::top("Ticks").show(&ctx, |ui| {
        ui.centered_and_justified(|ui| {
            let ticks_info = (1..=7)
                .map(|i| if i <= state.ticks { '+' } else { '-' })
                .collect::<String>();
            ui.label(RichText::new(ticks_info).size(25.));
        });
    });
    egui::TopBottomPanel::bottom("controls").show(&ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label("Controls:");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (key, value) in [
                        ("Esc", "Return to Main Menu"),
                        ("Bksp", "Clear text"),
                        ("F1", "Decrease WPM"),
                        ("F2", "Increase WPM"),
                        ("F3", "Decrease frequency"),
                        ("F4", "Increase frequency"),
                        ("F5", "Decrease volume"),
                        ("F6", "Increase volume"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:<8} - {}", key, value));
                        });
                    }
                });
                ui.vertical(|ui| {
                    ui.label("Settings:");
                    ui.horizontal(|ui| {
                        ui.label("WPM:");
                        let wpm = ui.add(egui::Slider::new(&mut state.wpm, MIN_WPM..=MAX_WPM));
                        if wpm.changed() {
                            state.normalize_values();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Frequency:");
                        ui.add(egui::Slider::new(
                            &mut state.frequency,
                            MIN_FREQUENCY..=MAX_FREQUENCY,
                        ));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Volume:");
                        ui.add(egui::Slider::new(
                            &mut state.volume,
                            MIN_VOLUME..=MAX_VOLUME,
                        ));
                    });
                });
            });
        });
    });
    // Main text area.
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            let buff = state.buffer.iter().collect::<String>();
            ui.label(egui::RichText::new(format!("{}{}|", state.text, buff)).size(32.));
        });
    });
    Ok(())
}

/// System that tracks morse timings.
///
/// Basically, every 3 ticks is a character boundary, and every 7 ticks is a word boundary.
pub fn timers(mut state: ResMut<WritingState>, delta: Res<Time>) {
    // Since each update might have different deltas,
    // we must calculate the tick based on the delta time.
    let Some(tick) = state.tick(delta.delta()) else {
        return;
    };

    // If the key is being pressed, do not do anything.
    if state.pressed {
        return;
    }

    if tick == 3 {
        if let Some(ch) = morse_to_char(&state.buffer.iter().collect::<String>()) {
            state.text.push(ch);
        }
        for (prosign, seq) in crate::consts::PROSIGNS.iter() {
            if &state.buffer.iter().collect::<String>() == seq {
                state.text.push_str(&prosign.to_string());
            }
        }
        // No matter if we found a value or not,
        // we need to clear up the buffer anyways.
        state.buffer.clear();
    } else if tick == 7 {
        if state.text.len() > 0 && !state.text.ends_with(' ') {
            state.text.push(' ');
        }
    }
}

/// This system syncs
/// volume values with current state.
///
/// Since values could have changed,
/// from UI.
fn sync_audio_values(
    state: Res<WritingState>,
    mut cmds: Commands,
    mut sine_asset: ResMut<Assets<SineAudio>>,
    sink_entity: Option<Single<(&mut AudioSink, Entity)>>,
) {
    // If it's none, we need to spawn it.
    let mut respawn_sound = sink_entity.is_none();
    for (_, asset) in sine_asset.iter_mut() {
        if asset.frequency != state.frequency as f32 {
            respawn_sound = true;
        }
    }
    if let Some(ent) = sink_entity {
        // We update volume on any change.
        let (mut sink, entity) = ent.into_inner();
        sink.set_volume(bevy::audio::Volume::Linear(state.volume as f32 * 0.01));
        // If entity needs to be respawned, despawn it.
        if respawn_sound {
            cmds.entity(entity).despawn();
        }
    }
    if respawn_sound {
        cmds.spawn((
            AudioPlayer(sine_asset.add(SineAudio {
                frequency: state.frequency as f32,
            })),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Linear(state.volume as f32 * 0.01),
                muted: true,
                ..PlaybackSettings::default()
            },
        ));
    }
}

pub fn sound_system(
    mut audio_sink: Single<&mut AudioSink>,
    mut messages: MessageReader<SoundEvent>,
) {
    audio_sink.is_muted();
    if let Some(event) = messages.read().last() {
        match event {
            SoundEvent::Start => {
                if audio_sink.is_muted() {
                    audio_sink.unmute();
                }
            }
            SoundEvent::Stop => {
                if !audio_sink.is_muted() {
                    audio_sink.mute();
                }
            }
        }
    }
}

pub fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WritingState>,
    mut app_state: ResMut<NextState<AppState>>,
    mut sound_queue: MessageWriter<SoundEvent>,
) {
    if keys.just_released(KeyCode::Space) {
        sound_queue.write(SoundEvent::Stop);
        state.pressed = false;
        if state.ticks <= 1 {
            state.buffer.push('.');
        } else {
            state.buffer.push('-');
        }
        state.reset_timer();
    } else if keys.just_pressed(KeyCode::Space) {
        sound_queue.write(SoundEvent::Start);
        state.pressed = true;
        state.reset_timer();
    } else if keys.just_pressed(KeyCode::F1) {
        state.wpm = state.wpm.saturating_sub(1);
    } else if keys.just_pressed(KeyCode::F2) {
        state.wpm = state.wpm.saturating_add(1);
    } else if keys.just_pressed(KeyCode::F3) {
        state.frequency = state.frequency.saturating_sub(50);
    } else if keys.just_pressed(KeyCode::F4) {
        state.frequency = state.frequency.saturating_add(50);
    } else if keys.just_pressed(KeyCode::F5) {
        state.volume = state.volume.saturating_sub(5);
    } else if keys.just_pressed(KeyCode::F6) {
        state.volume = state.volume.saturating_add(5);
    } else if keys.just_pressed(KeyCode::Backspace) {
        state.text.clear();
        state.buffer.clear();
    } else if keys.just_pressed(KeyCode::Escape) {
        app_state.set(AppState::MainMenu);
    }
}
