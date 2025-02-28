#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use binc::node_id::NodeId;
use binc::node_store::Node;
use bincgui::app::{create_toolbar, Application};
use eframe::egui::Ui;
use eframe::{egui, Storage};
use eframe::egui::UiKind::Frame;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Got issues",
        options,
        Box::new(|_cc| Ok(Box::new(IssuesApp::new()))),
    )
}

struct IssuesApp {
    application: Application,
    search_string: String,
    found_issues: Vec<NodeId>,
}

impl IssuesApp {
    fn new() -> Self {
        Self {
            application: Application::new(),
            search_string: String::new(),
            found_issues: vec![],
        }
    }

    fn update_search(&mut self) {
        self.found_issues = self.get_issues_for_search(&self.search_string);
    }

    fn get_issues_for_search(&self, search_string: &str) -> Vec<NodeId> {
        if !search_string.is_empty() {
            let terms = search_string.split(" ");

            if let Some(issue_id) = self.application.document.nodes.type_names.get_index("issue") {
                let mut issues = vec![];
                let summary_id = self.application.document.nodes.attribute_names.get_index("summary");

                for node in self.application.document.nodes.nodes() {
                    if Some(issue_id) == node.type_id {
                        if let Some(summary) = node.get_string_attribute(summary_id.unwrap()) {
                            let mut found = true;
                            let mut t = terms.clone();
                            while let Some(term) = t.next() {
                                if !summary.to_lowercase().contains(&term.to_lowercase()) {
                                    found = false;
                                    break;
                                }
                            }
                            if found {
                                issues.push(node.id);
                            }
                        }
                    }
                }
                return issues;
            }
        }
        vec![]
    }
}

impl eframe::App for IssuesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let summary_id = self.application.document.nodes.attribute_names.get_index("summary").unwrap_or(0);
        let permalink_id = self.application.document.nodes.attribute_names.get_index("permalink").unwrap_or(0);

        let frame = egui::Frame::default()
            .inner_margin(8.0)
            .fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar")
            .frame(frame)
            .show(ctx, |ui| {
                create_toolbar(&mut self.application, ui, |ui| {});
            });

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.vertical_centered(|ui| {
                if ui.text_edit_singleline(&mut self.search_string).changed() {
                    self.update_search();
                }

                let f = egui::Frame::default()
                    .inner_margin(16.0)
                    .fill(ctx.style().visuals.panel_fill)
                    .shadow(ctx.style().visuals.window_shadow)
                    .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke);


                for id in &self.found_issues {
                    f.show(ui, |ui| {
                        let node = self.application.document.nodes.get(*id).unwrap();
                        let label = node.get_string_attribute(summary_id).unwrap_or("?".to_string());
                        let url = node.get_string_attribute(permalink_id) .unwrap_or("?".to_string());
                        ui.hyperlink_to(label, url);
                    });
                }
        })
    });
}

fn save(&mut self, _storage: &mut dyn Storage) {
        // Save app state here
    }
}
