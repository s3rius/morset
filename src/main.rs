#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod audio;
mod consts;
mod inputs;
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
        Self {
            state: AppState::MainMenu,
            audio: None,
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
                    // We only create audio after user interaction.
                    // Otherwise, some browsers block audio playback.
                    self.audio = Some(AudioManager::new(600.0, 0.2).unwrap());
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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("MORSET"),
        ..Default::default()
    };
    tracing_subscriber::fmt().init();

    eframe::run_native(
        "MORSET",
        options,
        Box::new(|cc| Ok(Box::new(MorsetApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use eframe::web_sys;

    // Redirect `log` message to `console.log` and friends:
    tracing_subscriber::fmt()
        .with_writer(tracing_subscriber_wasm::MakeConsoleWriter::default())
        .with_max_level(tracing::Level::DEBUG)
        .without_time()
        .with_ansi(false)
        .init();

    tracing::info!("Starting MORSET web app");

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MorsetApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
