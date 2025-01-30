#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::fs::File;
use eframe::egui;
use binc::document::Document;
use binc::repository::Repository;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 300.0]),
        ..Default::default()
    };

    // Our application state:
    let app_name = "binc-demo";
    let mut document = Document::new(Repository::new());

    eframe::run_simple_native(app_name, options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(app_name);

            ui.menu_button("File", |ui| {
                if ui.button("Load Document").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Binc files", &["binc"]).pick_file() {
                        if let Ok(mut file) = File::open(path) {
                            if let Ok(repo) = Repository::read(&mut file) {
                                let doc = Document::new(repo);
                                println!("Loaded document with {} nodes", doc.node_count());
                                document = doc;
                            } else {
                                println!("Failed to read repository from file");
                            }
                        } else {
                            println!("Failed to open file");
                        }
                    }
                }
            });

            if ui.button("Show Document Contents").clicked() {
                for node in document.nodes.keys() {
                    ui.label(format!("Node: {:?}", node));
                }
            }
            if ui.button("ok").clicked()
            {
                println!("ok");
            }
        });
    })


}