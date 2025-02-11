#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::egui;
use eframe::egui::{Button, Context, Image, RichText, TextBuffer, Ui, Widget};
use binc::document::{AttributeValue, Node};
use uuid::Uuid;
use binc::change::Change;
use binc::util::shorten_uuid;
use gui::gui::*;

enum GuiAction {
    Undo,
    Redo,
    SelectNode { path: String },
    AddNode { path: String },
    RemoveNode { path: String },
    WrappedChange { change: Change },
    SetNodeExpanded { path: String, expanded: bool },
    ToggleSelectedNodeExpanded,
    /// Commit pending changes
    Commit,
    SelectPrevious,
    SelectNext,
    SelectParent,
    SelectFirstChild,
    ToggleEditing,
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

        check_keyboard(ctx, &app, &mut actions);

        let frame = egui::Frame::default().inner_margin(8.0).fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").default_width(200f32).show(ctx, |ui| {
            let selected_node =  { app.document.tree.get(&app.selected_node) };
            create_inspector(ui, &app.selected_node, selected_node, &mut app.selected_node_name, &mut actions);
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

fn check_keyboard(ctx: &Context, app: &SimpleApplication, mut actions: &mut Vec<GuiAction>) {
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

    if !app.is_editing {
        if !app.selected_node.is_empty() {
            let path = &app.selected_node;
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if app.expanded_nodes.contains(path) {
                    actions.push(GuiAction::SetNodeExpanded { path: path.clone(), expanded: false });
                } else {
                    actions.push(GuiAction::SelectParent);
                }
                actions.push(GuiAction::SetNodeExpanded { path: path.clone(), expanded: false });
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                actions.push(GuiAction::SetNodeExpanded { path: path.clone(), expanded: true });
            }
        }
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        actions.push(GuiAction::ToggleEditing);
    }
}

fn process_action(action: Option<GuiAction>, app: &mut SimpleApplication) {
    match action
    {
        Some(action) => {
            match action {
                GuiAction::SelectNode { path } => app.select_node(path),
                GuiAction::AddNode { path } => app.add_child(&path),
                GuiAction::RemoveNode { path } => app.remove_node(&path),
                GuiAction::Commit => app.commit(),
                GuiAction::WrappedChange { change } => app.document.add_and_apply_change(change),
                GuiAction::Undo => app.document.undo(),
                GuiAction::Redo => app.document.redo(),
                GuiAction::SelectPrevious => app.select_previous(),
                GuiAction::SelectNext => app.select_next(),
                GuiAction::SelectParent => app.select_parent(),
                GuiAction::SelectFirstChild => app.select_first_child(),
                GuiAction::SetNodeExpanded { path, expanded } => app.set_node_expanded(path, expanded),
                GuiAction::ToggleSelectedNodeExpanded => app.toggle_selected_node_expanded(),
                GuiAction::ToggleEditing => app.toggle_editing(),
            }
        }
        None => {}
    }
}

fn create_inspector(ui: &mut Ui, path: &String, node: Option<&Node>, node_name: &mut String, actions: &mut Vec<GuiAction>) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            egui::Grid::new("inspector_grid").num_columns(2).show(ui, |ui| {
                ui.label("Inspector");
                if ui.button("Delete Node").clicked() {
                    actions.push(GuiAction::RemoveNode { path: path.clone() });
                }
                ui.end_row();

                ui.label("name");
                if ui.text_edit_singleline(node_name).changed() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetString { path: path.clone(), attribute: "name".to_string(), value: node_name.clone() } });
                }
                ui.end_row();

                ui.label("Path");
                ui.label(path);
                ui.end_row();

                for (key, value) in &node.attributes {
                    ui.label(key);
                    ui.label(format!("{}", value));
                    ui.end_row();
                }
            });
        } else {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label("No selection");
                });
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
    create_node_tree(ui, &app.document.tree.root, app, actions, "");
}

fn create_node_tree(ui: &mut Ui, node: &Node, app: &SimpleApplication, actions: &mut Vec<GuiAction>, path: &str) {
     {
        let children = &node.children;
        let name = node.name.clone();
        let mut node_name = name.clone();
        let pretty_name = node.get_string_attribute("name");
        let label = if let Some(pretty_name) = pretty_name {
            pretty_name
        } else {
            name.clone()
        };
        let selected = app.selected_node == path && !path.is_empty();
        let mut text = RichText::new(label);
        if selected {
            text = text.color(ui.visuals().strong_text_color());
        }
        let is_expanded = app.expanded_nodes.contains(path);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let expand_icon = if is_expanded { "⏷" } else { "⏵" };
                if ui.label(expand_icon).on_hover_text("Expand/collapse node").clicked() {
                    actions.push(GuiAction::SetNodeExpanded { path: path.to_string(), expanded: !is_expanded });
                }

                let mut checked = node.get_bool_attribute("completed").unwrap_or(false);
                if ui.checkbox(&mut checked, "").clicked() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetBool { path: path.to_string(), attribute: "completed".to_string(), value: checked } });
                }

                if selected && app.is_editing {
                    let text_edit = ui.text_edit_singleline(&mut node_name);
                    text_edit.request_focus();
                    if text_edit.changed() {
                        actions.push(GuiAction::WrappedChange { change: Change::SetString { path: path.to_string(), attribute: "name".to_string(), value: node_name.clone() } });
                    }
                } else {
                    let label = ui.label(text);
                    if label.clicked() { actions.push(GuiAction::SelectNode { path: path.to_string() }); }
                    if label.double_clicked() { actions.push(GuiAction::ToggleEditing); }
                }

                ui.spacing();

                if selected {
                    if ui.label("✖").clicked() { actions.push(GuiAction::RemoveNode { path: path.to_string() }); }
                }
            });

            if is_expanded {
                ui.indent(path, |ui| {
                    for child in children {
                        let sub_path = path.to_string() + "/" + &child.name;
                        create_node_tree(ui, child, app, actions, path);
                    }
                    let add_button = ui.label("⊞").on_hover_text("Add child node");
                    if add_button.clicked() {
                        let new_path = path.to_string() + "/new";
                        actions.push(GuiAction::AddNode { path: new_path });
                    }
                });
            }
        });
    }
}

fn get_label(id_string: String, name: &String) -> String {
    if !name.is_empty() {
        let name = name;
        format!("{}", name)
    } else { id_string }
}