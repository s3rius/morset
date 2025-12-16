#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::time::Instant;

mod audio;
mod consts;
mod screens;
mod state;
mod utils;

use audio::AudioManager;
use state::AppState;

/// Main application structure
struct MorsetApp {
    state: AppState,
    audio: Option<AudioManager>,
    main_menu: screens::MainMenuScreen,
    writing_screen: Option<screens::WritingScreen>,
    last_update: Instant,
}

impl MorsetApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize audio manager
        let audio = AudioManager::new(550.0, 0.2).ok();
        
        Self {
            state: AppState::MainMenu,
            audio,
            main_menu: screens::MainMenuScreen::new(),
            writing_screen: None,
            last_update: Instant::now(),
        }
    }
}

impl eframe::App for MorsetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;
        
        match self.state {
            AppState::MainMenu => {
                if let Some(new_state) = self.main_menu.render(ctx) {
                    self.state = new_state;
                    // Initialize writing screen when entering
                    if self.state == AppState::Writing {
                        self.writing_screen = Some(screens::WritingScreen::new());
                    }
                }
            }
            AppState::Writing => {
                if let Some(ref mut screen) = self.writing_screen
                    && let Some(new_state) = screen.update(ctx, delta, &mut self.audio)
                {
                    self.state = new_state;
                    // Clean up when leaving
                    if self.state != AppState::Writing {
                        self.writing_screen = None;
                    }
                }
            }
            AppState::Listening => {
                // TODO: Implement listening screen
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Listening mode - Coming soon!");
                    if ui.button("Back to Menu").clicked() {
                        self.state = AppState::MainMenu;
                    }
                });
            }
        }
        
        // Request continuous repaint for smooth updates
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("MORSET"),
        ..Default::default()
    };
    
    eframe::run_native(
        "MORSET",
        options,
        Box::new(|cc| Ok(Box::new(MorsetApp::new(cc)))),
    )
}
