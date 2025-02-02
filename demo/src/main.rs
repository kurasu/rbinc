#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::egui;
use eframe::egui::Ui;
use binc::document::Node;
use uuid::Uuid;
use gui::gui::*;

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 300.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Demo", options, move |ctx, _frame| {

        egui::CentralPanel::default().show(ctx, |ui| {
            create_toolbar(&mut app, ui);
            create_tree(ui, &mut app);
        });
        egui::SidePanel::right("inspector_panel").show(ctx, |ui| {
            let selected_node = if let Some(id) = &app.selected_node { app.document.nodes.get(id) } else { None };
            create_inspector(ui, selected_node);
        });
    })
}

fn create_inspector(ui: &mut Ui, node: Option<&Node>) {
    if let Some(node) = node {
        ui.label("Inspector");
        for (key, value) in &node.attributes {
            ui.label(format!("{}: {:?}", key, attribute_value_to_string(value.as_ref())));
        }
    }
    else {
        ui.label("No node selected");
    }
}

fn attribute_value_to_string(value: &dyn Any) -> String {
    if let Some(value) = value.downcast_ref::<String>() {
        value.clone()
    } else if let Some(value) = value.downcast_ref::<&str>() {
        value.to_string()
    } else if let Some(value) = value.downcast_ref::<bool>() {
        value.to_string()
    } else if let Some(value) = value.downcast_ref::<i32>() {
        value.to_string()
    } else if let Some(value) = value.downcast_ref::<f64>() {
        value.to_string()
    } else {
        "None".to_string()
    }
}

fn create_tree(ui: &mut Ui, app: &mut SimpleApplication) {
    for root in app.roots.clone() {
        create_node_tree(ui, &root, app);
    }
}

fn create_node_tree(ui: &mut Ui, node_id: &Uuid, app: &mut SimpleApplication) {
    if let Some(node) = app.document.nodes.get(node_id) {
        let children = &node.children.clone();
        let id_string = format!("ID: {:?}", node_id);
        let name = node.attributes.get("name");
        let label = if let Some(name) = name {
            let name = name.downcast_ref::<String>().unwrap();
            format!("{}", name)
        }
        else { id_string };

        if ui.collapsing(label, |ui| {
            for child_id in children {
                create_node_tree(ui, child_id, app);
            }
        }).header_response.clicked() {
            // Select the node
            app.selected_node = Some(*node_id);
        }
    }
}