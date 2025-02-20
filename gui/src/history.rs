use crate::app::{Application, GuiAction};
use eframe::egui::Ui;

pub struct History {
    pub show_history: bool,
    commit_message: String,
}

impl History {
    pub fn new() -> Self {
        Self {
            show_history: false,
            commit_message: String::new(),
        }
    }

    pub fn create_history(
        &mut self,
        ui: &mut Ui,
        app: &mut Application,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        for revision in &app.document.repository.revisions {
            let label = format!(
                "{} by {} on {}",
                revision.message, revision.user_name, revision.date
            );
            if ui
                .collapsing(label, |ui| {
                    for change in &revision.changes {
                        ui.label(change.to_string());
                    }
                })
                .header_response
                .clicked()
            {
                // Handle revision selection if needed
            }
        }

        let pending = &app.document.pending_changes;
        if !pending.changes.is_empty() {
            ui.collapsing("Pending changes", |ui| {
                ui.text_edit_singleline(&mut self.commit_message);

                if ui.button("Snapshot").clicked() {
                    on_action(GuiAction::Commit {
                        message: self.commit_message.clone(),
                    });
                }
                for change in &pending.changes {
                    ui.label(change.to_string());
                }
                app.document.undo_changes.iter().rev().for_each(|change| {
                    ui.colored_label(ui.visuals().weak_text_color(), change.to_string());
                });
            });
        }
        ui.allocate_space(ui.available_size());
    }
}
