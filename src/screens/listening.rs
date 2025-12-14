use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::state::AppState;

pub struct ListeningScreenPlugin;

impl Plugin for ListeningScreenPlugin {
    fn build(&self, app: &mut App) {}
}

struct ListeningState {
    pub wpm: u8,
    pub frequency: usize,
    pub volume: usize,
}

fn setup_egui_main_menu(
    mut contexts: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    mut exit_writer: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {});
    });
    Ok(())
}
