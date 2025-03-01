#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use binc::builder::NodeBuilder;
use binc::document::Document;
use binc::node_id::NodeId;
use binc::node_store::Node;
use bincgui::app::{create_toolbar, Application, GuiAction};
use eframe::egui::{ComboBox, Ui};
use eframe::{egui, App, CreationContext, Storage};
use std::fs::File;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Notes",
        options,
        Box::new(|cc| Ok(NotesApp::new_or_from_storage(cc))),
    )
}

struct NotesApp {
    application: Application,
}

impl NotesApp {
    fn new_or_from_storage(cc: &CreationContext) -> Box<dyn App> {
        if let Some(storage) = cc.storage {
            if let Some(path) = storage.get_string("document_path") {
                if let Ok(mut file) = File::open(&path) {
                    if let Ok(doc) = Document::read(&mut file) {
                        let mut app = NotesApp::new();
                        app.application.set_document(doc);
                        return Box::new(app);
                    }
                }
            }
        }
        Box::new(NotesApp::new())
    }

    fn new() -> Self {
        let mut d = Document::default();
        let l1 = d.add_node(NodeId::ROOT_NODE);
        d.set_node_type(l1, "list");
        d.set_node_name(l1, "My List");
        let t1 = d.add_node(l1);
        d.set_node_type(t1, "task");
        d.set_node_name(t1, "start");

        let mut app = Self {
            application: Application::new_with_document(d),
        };
        app.application.ui.root = l1;
        app
    }

    fn child_nodes(&self, ui: &mut Ui, node: &Node) {
        for n in &node.children {
            self.node(ui, self.get_node(n.clone()));
        }
    }

    fn get_name(&self, id: NodeId) -> &str {
        self.get_node(id).get_name().unwrap_or("?")
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

            /*if !node.children.is_empty() && self.application.is_node_expanded(node.id) {
                ui.indent(node.id, |ui| {
                    self.child_nodes(ui, node);
                });
            }*/
        });
    }

    fn get_lists(&self) -> Vec<NodeId> {
        let list_type = self
            .application
            .document
            .nodes
            .type_names
            .get_index("list")
            .expect("type list not present");

        let mut lists = self
            .application
            .get(NodeId::ROOT_NODE)
            .unwrap()
            .children
            .clone();

        lists
            .retain(|id| self.application.get(*id).expect("Must exist").type_id == Some(list_type));
        lists
    }

    fn perform_action(&mut self, action: GuiAction) {
        self.application.process_action(action)
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
                create_toolbar(&mut self.application, ui, |_ui| {});
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let root = self.application.root();

            let f = egui::Frame::default()
                .inner_margin(16.0)
                .fill(ctx.style().visuals.panel_fill)
                .shadow(ctx.style().visuals.window_shadow)
                .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke);

            let lists = self.get_lists().clone();
            let root = self.application.ui.root;

            ui.vertical_centered(|ui| {
                f.show(ui, |ui| {
                    let label = self.get_name(root);
                    ComboBox::new("lists-combo", "")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            for list in lists {
                                if ui
                                    .selectable_label(list == root, self.get_name(list))
                                    .clicked()
                                {
                                    self.perform_action(GuiAction::SetRootNode { node: list })
                                }
                            }
                            if ui.label("+ new list").clicked() {
                                // Create new list
                            }
                        });

                    self.child_nodes(ui, self.get_node(root));
                })
            });
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        if let Some(path) = &self.application.document_path {
            storage.set_string("document_path", path.to_str().unwrap().to_string());
        }
    }
}
