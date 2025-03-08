use crate::app::GuiAction;
use binc::journal::Journal;
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
        journal: &Journal,
        undo_revision: Option<usize>,
        _on_action: &mut impl FnMut(GuiAction),
    ) {
        let to = undo_revision.unwrap_or(journal.operations.len());

        if undo_revision.is_some() {
            journal.operations[to..]
                .iter()
                .rev()
                .take(30)
                .for_each(|change| {
                    ui.weak(change.to_string());
                });
            ui.separator();
        }

        journal.operations[..to]
            .iter()
            .rev()
            .take(100)
            .for_each(|change| {
                ui.label(change.to_string());
            });
    }
}
