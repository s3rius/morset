#[cfg(not(target_arch = "wasm32"))]
use bevy::window::WindowResolution;
use bevy::{audio::AddAudioSource, prelude::*};
use bevy_egui::EguiPlugin;

use crate::state::AppState;

mod consts;
mod screens;
mod sine_audio;
mod state;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                #[cfg(not(target_arch = "wasm32"))]
                primary_window: Some(Window {
                    title: String::from("MORSET"),
                    mode: bevy::window::WindowMode::Windowed,
                    resolution: WindowResolution::new(1280, 720),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                #[cfg(target_arch = "wasm32")]
                primary_window: Some(Window {
                    canvas: Some(String::from("#gameboard")),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
        ))
        .init_state::<AppState>()
        .add_audio_source::<sine_audio::SineAudio>()
        .add_systems(Startup, setup_camera_system)
        .add_plugins(screens::MainMenuPlugin)
        .add_plugins(screens::WritingScreenPlugin)
        .add_plugins(screens::ListeningScreenPlugin)
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}
