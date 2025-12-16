use crate::state::AppState;

#[allow(dead_code)]
pub struct ListeningScreen;

#[allow(dead_code)]
impl ListeningScreen {
    pub fn new() -> Self {
        Self
    }

    /// Render the listening screen (placeholder for now)
    pub fn render(&mut self, ctx: &egui::Context) -> Option<AppState> {
        let mut new_state = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Listening Mode");
                ui.label("Coming soon!");
                ui.add_space(20.0);

                if ui.button("Back to Menu").clicked() {
                    new_state = Some(AppState::MainMenu);
                }
            });
        });

        new_state
    }
}
