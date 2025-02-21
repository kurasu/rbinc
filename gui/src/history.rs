use crate::app::{Application, GuiAction};
use eframe::egui::Ui;

pub struct History {
    pub show_history: bool,
    snapshot_message: String,
}

impl History {
    pub fn new() -> Self {
        Self {
            show_history: false,
            snapshot_message: String::new(),
        }
    }

    pub fn create_history(
        &mut self,
        ui: &mut Ui,
        app: &mut Application,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        for change in &app.document.repository.changes {
            ui.label(change.to_string());
        }
    }
}
