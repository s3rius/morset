use egui::{InputState, Key};

pub trait InputStateExt {
    fn key_just_pressed(&self, desired_key: Key) -> bool;
}

impl InputStateExt for InputState {
    // This function checks if a specific key was just pressed (not held down)
    // by filtering the events for key press events with repeat field set to true.
    fn key_just_pressed(&self, desired_key: Key) -> bool {
        self.events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    egui::Event::Key { key, pressed: true, repeat: false, .. }
                    if *key == desired_key
                )
            })
            .count()
            > 0
    }
}
