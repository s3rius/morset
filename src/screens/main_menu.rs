use egui::{self, RichText};

use crate::state::AppState;

pub struct MainMenuScreen;

impl MainMenuScreen {
    pub fn new() -> Self {
        Self
    }

    /// Render the main menu and return the new state if changed
    pub fn render(&mut self, ctx: &egui::Context) -> Option<AppState> {
        let mut new_state = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.set_max_width(ui.max_rect().width() / 2.);
                ui.add_space(200.0);

                ui.heading(RichText::new("MORSET").size(48.0));
                ui.add_space(20.0);
                ui.label("Morse Code Practice");
                ui.add_space(20.0);

                if ui.button(RichText::new("Writing").size(24.0)).clicked() {
                    new_state = Some(AppState::Writing);
                }

                ui.add_space(10.0);
                if ui.button(RichText::new("Listening").size(24.0)).clicked() {
                    new_state = Some(AppState::Listening);
                }

                ui.add_space(10.0);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button(RichText::new("Exit").size(24.0)).clicked() {
                        std::process::exit(0);
                    }
                }
            });
        });

        new_state
    }
}
