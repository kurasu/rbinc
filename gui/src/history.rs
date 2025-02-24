use crate::app::GuiAction;
use binc::repository::Repository;
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
        &self,
        ui: &mut Ui,
        repository: &Repository,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        for change in &repository.changes {
            ui.label(change.to_string());
        }
    }
}
