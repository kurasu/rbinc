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
        undo_revision: Option<usize>,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        repository
            .changes
            .iter()
            .enumerate()
            .rev()
            .take(100)
            .for_each(|(index, change)| {
                if undo_revision.is_some() && index >= undo_revision.unwrap() {
                    ui.weak(change.to_string());
                } else {
                    ui.label(change.to_string());
                }
            });
    }
}
