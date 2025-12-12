mod consts;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0]) // wide enough for the drag-drop overlay text
            .with_title("Morset"),
        ..Default::default()
    };
    eframe::run_native(
        "Morset",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )?;
    Ok(())
}

#[derive(Default)]
pub struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::default().with_main_align(egui::Align::Center),
                |ui| {
                    ui.vertical_centered(|ui| {
                        // ui.horizontal_centered(|ui| {
                            ui.label("Morset");
                            ui.label("What's up?");
                        // })
                    })
                },
            );
        });
    }
}
