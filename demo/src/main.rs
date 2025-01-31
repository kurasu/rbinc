#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use eframe::egui::Ui;
use binc::document::Document;
use uuid::Uuid;
use crate::gui::*;

mod gui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 300.0]),
        ..Default::default()
    };

    // Our application state:
    let app_name = "BINC Demo";
    let mut document = new_document();

    eframe::run_simple_native(app_name, options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {
                if ui.button("New").clicked() {

                    document = new_document();
                }
                if ui.button("Open").clicked() {
                    let result = open_document();
                    if let Ok(Some(result)) = result {
                        document = result;
                    }
                    else { show_error(result, "Failed to open document"); }
                }
                if ui.button("Save").clicked() {
                    save_document(&document);
                }
            });

            create_tree(ui, &document);
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