#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::fs::File;
use std::io;
use eframe::egui;
use rfd::MessageLevel::Error;
use binc::document::Document;
use binc::repository::Repository;

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
            for node in document.nodes.keys() {
                ui.label(format!("Node: {:?}", node));
            }
        });
    })
}


fn show_error<T>(result: io::Result<T>, description: &str) {
    if let Err(error) = result {
        let text = format!("{}\n\n{}", description.to_string(), error.to_string());
        rfd::MessageDialog::new().set_level(Error).set_title("Error").set_description(text).show();
    }
}

fn open_document() -> io::Result<Option<Document>> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).pick_file();

    if let Some(path) = path {
        let mut file = File::open(path)?;
        let document = Document::read(&mut file)?;
        return Ok(Some(document));
    }

    Ok(None)
}

fn save_document(document: &Document) -> io::Result<bool> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).save_file();

    if let Some(path) = path {
        let mut file = File::create(path)?;
        document.write(&mut file)?;
        return Ok(true);
    }
    Ok(false)
}

fn new_document() -> Document {
    Document::new(Repository::new())
}