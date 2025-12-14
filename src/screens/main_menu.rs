use bevy::{log::tracing, prelude::*};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::state::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            setup_egui_main_menu.run_if(in_state(AppState::MainMenu)),
        );
    }
}

fn setup_egui_main_menu(
    mut contexts: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    mut exit_writer: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            ui.group(|ui| {
                if ui.button("Writing").clicked() {
                    tracing::info!("Starting writing");
                    state.set(AppState::Writing);
                }
                // if ui.button("Listening").clicked() {
                //     tracing::info!("Starting listening");
                //     state.set(AppState::Writing);
                // }
                if ui.button("Exit").clicked() {
                    exit_writer.write(AppExit::Success);
                }
            });
        });
    });
    Ok(())
}
