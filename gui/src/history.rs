use eframe::egui::Ui;
use crate::app::{Application, GuiAction};


pub fn create_history(ui: &mut Ui, app: &Application, on_action: &mut impl FnMut(GuiAction)) {
    for revision in &app.document.repository.revisions {
        let label = format!("{} by {} on {}", revision.message, revision.user_name, revision.date);
        if ui.collapsing(label, |ui| {
            for change in &revision.changes {
                ui.label(change.to_string());
            }
        }).header_response.clicked() {
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
    });
    ui.allocate_space(ui.available_size());
}
