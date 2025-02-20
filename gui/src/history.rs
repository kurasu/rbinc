use crate::app::{Application, GuiAction};
use eframe::egui::Ui;

pub fn create_history(ui: &mut Ui, app: &Application, on_action: &mut impl FnMut(GuiAction)) {
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
    ui.collapsing("Pending changes", |ui| {
        if ui.button("Snapshot").clicked() {
            on_action(GuiAction::Commit);
        }
        for change in &pending.changes {
            ui.label(change.to_string());
        }
        app.document.undo_changes.iter().rev().for_each(|change| {
            ui.colored_label(ui.visuals().weak_text_color(), change.to_string());
        });
    });
    ui.allocate_space(ui.available_size());
}
