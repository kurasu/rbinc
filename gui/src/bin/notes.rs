#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use binc::node_id::NodeId;
use binc::node_store::Node;
use bincgui::app::{create_toolbar, Application};
use eframe::egui::Ui;
use eframe::{egui, Storage};

struct NotesApp {
    application: Application,
}

impl NotesApp {
    fn new() -> Self {
        Self {
            application: Application::new(),
        }
    }

    fn child_nodes(&self, ui: &mut Ui, node: &Node) {
        for n in &node.children {
            self.node(ui, self.get_node(n.clone()));
        }
    }

    fn get_node(&self, id: NodeId) -> &Node {
        self.application.get(id).expect("Node not found")
    }

    fn node(&self, ui: &mut Ui, node: &Node) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.application.is_node_expanded(node.id), "");
                let label = match node.get_name() {
                    Some(name) => format!("{}", name),
                    None => format!("ID{}", node.id),
                };
                ui.label(label);
            });

            if !node.children.is_empty() && self.application.is_node_expanded(node.id) {
                ui.indent(node.id, |ui| {
                    self.child_nodes(ui, node);
                });
            }
        });
    }
}

impl eframe::App for NotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::default()
            .inner_margin(8.0)
            .fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar")
            .frame(frame)
            .show(ctx, |ui| {
                create_toolbar(&mut self.application, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let root = self.application.root();

            let f = egui::Frame::default()
                .inner_margin(16.0)
                .fill(ctx.style().visuals.panel_fill)
                .shadow(ctx.style().visuals.window_shadow)
                .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke);

            ui.vertical_centered(|ui| {
                f.show(ui, |ui| {
                    ui.heading("Notes");
                    self.child_nodes(ui, root);
                })
            });
        });
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        // Save app state here
    }
}

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Notes",
        options,
        Box::new(|_cc| Ok(Box::new(NotesApp::new()))),
    )
}
