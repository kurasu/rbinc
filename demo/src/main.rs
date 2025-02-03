#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::egui;
use eframe::egui::Ui;
use binc::document::Node;
use uuid::Uuid;
use binc::changes::shorten_uuid;
use gui::gui::*;

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Demo", options, move |ctx, _frame| {
        let frame = egui::Frame::default().inner_margin(8.0).fill(egui::Color32::from_gray(36));
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").show(ctx, |ui| {
            let selected_node = if let Some(id) = &app.selected_node { app.document.nodes.get(id) } else { None };
            create_inspector(ui, selected_node);
        });
        egui::TopBottomPanel::bottom("history_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                create_history(ui, &app);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            create_tree(ui, &mut app);
        });
    })
}


fn create_inspector(ui: &mut Ui, node: Option<&Node>) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            ui.label("Inspector");
            egui::Grid::new("inspector_grid").show(ui, |ui| {
                ui.label("name");
                ui.text_edit_singleline(&mut "".to_string());
                ui.end_row();
                for (key, value) in &node.attributes {
                    ui.label(key);
                    ui.label(format!("{:?}", attribute_value_to_string(value.as_ref())));
                    ui.end_row();
                }
            });
        } else {
            ui.label("No node selected");
        }
    });
}

fn create_history(ui: &mut Ui, app: &SimpleApplication) {
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
        let id_string = format!("ID: {:?}", shorten_uuid(*node_id));
        let name = node.attributes.get("name");
        let label = get_label(id_string, name);

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

fn get_label(id_string: String, name: Option<&Box<dyn Any>>) -> String {
    if let Some(name) = name {
        let name = name.downcast_ref::<String>().unwrap();
        format!("{}", name)
    } else { id_string }
}