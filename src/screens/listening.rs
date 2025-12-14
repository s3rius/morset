use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::state::AppState;

pub struct ListeningScreenPlugin;

impl Plugin for ListeningScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            setup_egui_main_menu.run_if(in_state(AppState::Listening)),
        )
        .add_systems(OnEnter(AppState::Listening), init_screen);
    }
}

#[derive(Resource, Debug)]
struct ListeningState {
    wpm: u8,
    frequency: usize,
    volume: usize,
}

fn setup_egui_main_menu(
    mut contexts: EguiContexts,
    _state: ResMut<NextState<AppState>>,
    _exit_writer: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered_justified(|_ui| {});
    });
    Ok(())
}

fn init_screen(mut cmds: Commands) {
    let state = ListeningState {
        wpm: 10,
        frequency: 550,
        volume: 50,
    };
    cmds.insert_resource(state);
}
