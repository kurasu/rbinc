#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::egui;
use eframe::egui::{Color32, RichText, Ui};
use binc::document::{AttributeValue, Node};
use uuid::Uuid;
use binc::util::shorten_uuid;
use gui::gui::*;
use crate::GuiAction::AddNode;

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Demo", options, move |ctx, _frame| {
        let mut action: Option<GuiAction> = None;

        let frame = egui::Frame::default().inner_margin(8.0).fill(egui::Color32::from_gray(36));
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").show(ctx, |ui| {
            let selected_node = if let Some(id) = &app.selected_node { app.document.nodes.get(id) } else { None };
            action = create_inspector(ui, selected_node);
        });
        egui::TopBottomPanel::bottom("history_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if action.is_none() {
                    action = create_history(ui, &app);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if action.is_none() {
                action = create_tree(ui, &mut app);
            }
        });

        process_action(action, &mut app);
    })
}

fn process_action(action: Option<GuiAction>, app: &mut SimpleApplication) {
    match action
    {
        Some(action) => {
            match action {
                GuiAction::SelectNode { node } => {
                    app.selected_node = Some(node);
                }
                GuiAction::AddNode { parent, index } => {
                    app.add_child(&parent, index);
                }
                GuiAction::RemoveNode { node } => {
                    app.remove_node(&node);
                }
                GuiAction::Commit => {
                    app.commit();
                }
            }
        }
        None => {}
    }
}

fn create_inspector(ui: &mut Ui, node: Option<&Node>) -> Option<GuiAction> {
    let mut action : Option<GuiAction> = None;
    ui.vertical(|ui| {
        if let Some(node) = node {
            ui.label("Inspector");
            egui::Grid::new("inspector_grid").show(ui, |ui| {
                ui.label("name");
                ui.text_edit_singleline(&mut "".to_string());
                ui.end_row();

                ui.label("UUID");
                ui.label(shorten_uuid(&node.uuid));
                ui.end_row();

                for (key, value) in &node.attributes {
                    ui.label(key);
                    ui.label(format!("{}", value));
                    ui.end_row();
                }
                if ui.button("Delete Node").clicked() {
                    action = Some(GuiAction::RemoveNode { node: node.uuid });
                }
                ui.end_row();
            });
        } else {
            ui.label("No node selected");
        }
    });
    action
}

fn create_history(ui: &mut Ui, app: &SimpleApplication) -> Option<GuiAction> {
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
    let pending = &app.document.pending_changes;
    let collapsing = ui.collapsing("Pending", |ui| {
        if ui.button("Commit").clicked() {
            return Some(GuiAction::Commit);
        }
        for change in &pending.changes {
            ui.label(change.to_string());
        }
        None
    });
    if let Some(action) = collapsing.body_returned {
        if action.is_some() {
            return action
        }
    }
    None
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

enum GuiAction {
    SelectNode { node: Uuid },
    AddNode { parent: Uuid, index: u64 },
    RemoveNode { node: Uuid },
    Commit // Commit pending changes
}

fn create_tree(ui: &mut Ui, app: &mut SimpleApplication) -> Option<GuiAction> {
    let mut action: Option<GuiAction> = None;
    for root in app.roots.clone() {
        action = create_node_tree(ui, &root, app);
    }

    return action
}

fn create_node_tree(ui: &mut Ui, node_id: &Uuid, app: &SimpleApplication) -> Option<GuiAction>{
    if let Some(node) = app.document.nodes.get(node_id) {
        let children = &node.children.clone();
        let id_string = format!("ID: {:?}", shorten_uuid(node_id));
        let name = node.attributes.get("name");
        let label = get_label(id_string, name);
        let selected = app.selected_node == Some(*node_id);
        let mut text = RichText::new(label);
        if selected {
            text = text.color(Color32::YELLOW);
        }

        let collapsing_response = ui.collapsing(text, |ui| {
            let mut index = 0u64;
            for child_id in children {
                let result = create_node_tree(ui, child_id, app);
                if result.is_some() {
                    return result
                }
                index += 1;
            }
            let add_button = ui.button("+").on_hover_text("Add child node");
            if add_button.clicked() {
                let action = AddNode { parent: *node_id, index };
                Some(action)
            } else { None }
        });
        if collapsing_response.header_response.clicked() {
            return Some(GuiAction::SelectNode { node: *node_id })
        }
        else if let Some(action) = collapsing_response.body_returned {
            if action.is_some() {
                return action
            }
        }
    }
    None
}

fn get_label(id_string: String, name: Option<&AttributeValue>) -> String {
    if let Some(name) = name {
        let name = name;
        format!("{}", name)
    } else { id_string }
}