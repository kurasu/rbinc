#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::{egui, Storage};
use crate::app::Application;

struct NotesApp {
    application: Application
}

impl NotesApp {
    fn new() -> Self {
        Self {
            application: Application::new()
        }
    }
}

impl eframe::App for NotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Notes");
            ui.label("This is a work in progress.");
        });
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        // Save app state here
    }
}

pub fn main() -> eframe::Result {
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