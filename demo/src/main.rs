#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::egui;
use eframe::egui::{Color32, Context, RichText, Ui};
use binc::document::{AttributeValue, Node};
use uuid::Uuid;
use binc::change::Change;
use binc::util::shorten_uuid;
use gui::gui::*;

enum GuiAction {
    Undo,
    Redo,
    SelectNode { node: Uuid },
    AddNode { parent: Uuid, index: u64 },
    RemoveNode { node: Uuid },
    WrappedChange { change: Change },
    /// Commit pending changes
    Commit,
    SelectPrevious,
    SelectNext,
    SelectParent,
    SelectFirstChild,
}

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Demo", options, move |ctx, _frame| {
        let mut actions: Vec<GuiAction> = vec![];

        check_keyboard(ctx, &mut actions);

        let frame = egui::Frame::default().inner_margin(8.0).fill(egui::Color32::from_gray(36));
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").default_width(200f32).show(ctx, |ui| {
            let selected_node = if let Some(id) = &app.selected_node { app.document.nodes.get(id) } else { None };
            create_inspector(ui, selected_node, &mut app.selected_node_name, &mut actions);
        });
        egui::TopBottomPanel::bottom("history_panel").default_height(160f32).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                create_history(ui, &app, &mut actions);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            create_tree(ui, &mut app, &mut actions);
        });

        for action in actions {
            process_action(Some(action), &mut app);
        }
    })
}

fn check_keyboard(ctx: &Context, mut actions: &mut Vec<GuiAction>) {
    if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
        actions.push(GuiAction::Undo);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.command) {
        actions.push(GuiAction::Redo);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        actions.push(GuiAction::SelectPrevious);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        actions.push(GuiAction::SelectNext);
    }
    if (ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) || ctx.input(|i| i.key_pressed(egui::Key::Backspace))) {
        actions.push(GuiAction::SelectParent);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
        actions.push(GuiAction::SelectFirstChild);
    }
}

fn process_action(action: Option<GuiAction>, app: &mut SimpleApplication) {
    match action
    {
        Some(action) => {
            match action {
                GuiAction::SelectNode { node } => app.select_node(Some(node)),
                GuiAction::AddNode { parent, index } => app.add_child(&parent, index),
                GuiAction::RemoveNode { node } => app.remove_node(&node),
                GuiAction::Commit => app.commit(),
                GuiAction::WrappedChange { change } => app.document.add_and_apply_change(change),
                GuiAction::Undo => app.document.undo(),
                GuiAction::Redo => app.document.redo(),
                GuiAction::SelectPrevious => app.select_previous_sibling(),
                GuiAction::SelectNext => app.select_next_sibling(),
                GuiAction::SelectParent => app.select_parent(),
                GuiAction::SelectFirstChild => app.select_first_child(),
            }
        }
        None => {}
    }
}

fn create_inspector(ui: &mut Ui, node: Option<&Node>, node_name: &mut String, actions: &mut Vec<GuiAction>) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            egui::Grid::new("inspector_grid").num_columns(2).show(ui, |ui| {
                ui.label("Inspector");
                if ui.button("Delete Node").clicked() {
                    actions.push(GuiAction::RemoveNode { node: node.uuid });
                }
                ui.end_row();

                ui.label("name");
                if ui.text_edit_singleline(node_name).changed() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetString { node: node.uuid, attribute: "name".to_string(), value: node_name.clone() } });
                }
                ui.end_row();

                ui.label("UUID");
                ui.label(node.uuid.to_string());
                ui.end_row();

                for (key, value) in &node.attributes {
                    ui.label(key);
                    ui.label(format!("{}", value));
                    ui.end_row();
                }
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.label("No node selected");
            });
        }
    });
}

fn create_history(ui: &mut Ui, app: &SimpleApplication, actions: &mut Vec<GuiAction>) {
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
    ui.collapsing("Pending", |ui| {
        if ui.button("Commit").clicked() {
            actions.push(GuiAction::Commit);
        }
        for change in &pending.changes {
            ui.label(change.to_string());
        }
    });
    ui.allocate_space(ui.available_size());
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

fn create_tree(ui: &mut Ui, app: &mut SimpleApplication, actions: &mut Vec<GuiAction>) {
    for root in app.roots.clone() {
        create_node_tree(ui, &root, app, actions);
    }
}

fn create_node_tree(ui: &mut Ui, node_id: &Uuid, app: &SimpleApplication, actions: &mut Vec<GuiAction>) {
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
                create_node_tree(ui, child_id, app, actions);
                index += 1;
            }
            let add_button = ui.button("+").on_hover_text("Add child node");
            if add_button.clicked() {
                actions.push(GuiAction::AddNode { parent: *node_id, index });
            }
        });
        if collapsing_response.header_response.clicked() {
            actions.push(GuiAction::SelectNode { node: *node_id })
        }
    }
}

fn get_label(id_string: String, name: Option<&AttributeValue>) -> String {
    if let Some(name) = name {
        let name = name;
        format!("{}", name)
    } else { id_string }
}