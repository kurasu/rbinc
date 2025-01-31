#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use eframe::egui::Ui;
use binc::document::Document;
use uuid::Uuid;
use gui::gui::*;

fn main() -> eframe::Result {
    let mut app = SimpleApplication { document: Box::from(new_document()),
        view: |ui, app| {
            create_toolbar(app, ui);
            create_tree(ui, &app.document);
        }
    };
    create_simple_application(app, "BINC Demo")
}

fn create_simple_application(mut app: SimpleApplication, app_name: &str) -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 300.0]),
        ..Default::default()
    };

    eframe::run_simple_native(app_name, options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            (app.view)(ui, &mut app);
        });
    })
}

fn create_tree(ui: &mut Ui, document: &Document) {
    for node_id in document.nodes.iter() {
        create_node_tree(ui, node_id.0, document);
        return;
    }
}

fn create_node_tree(ui: &mut Ui, node_id: &Uuid, document: &Document) {
    if let Some(node) = document.nodes.get(node_id) {
        let id_string = format!("ID: {:?}", node_id);
        let name = node.attributes.get("name");
        let label = if let Some(name) = name {
            let name = name.downcast_ref::<String>().unwrap();
            format!("{}", name)
        }
        else { id_string };

        ui.collapsing(label, |ui| {
            for child_id in &node.children {
                create_node_tree(ui, child_id, document);
            }
        });
    }
}