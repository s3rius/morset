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
                ui.add_space(200.0);
                
                ui.heading(RichText::new("MORSET").size(48.0));
                ui.add_space(20.0);
                ui.label("Morse Code Practice");
                ui.add_space(40.0);
                
                ui.group(|ui| {
                    if ui.button(RichText::new("Writing").size(24.0)).clicked() {
                        new_state = Some(AppState::Writing);
                    }
                    
                    if ui.button(RichText::new("Listening").size(24.0)).clicked() {
                        new_state = Some(AppState::Listening);
                    }
                    
                    if ui.button(RichText::new("Exit").size(24.0)).clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
        
        new_state
    }
}
