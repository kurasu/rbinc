#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use eframe::egui::{Context, Frame, Id, RichText, Ui};
use binc::change::Change;
use binc::node_id::NodeId;
use binc::node_store::Node;
use crate::app::{create_toolbar, SimpleApplication};

pub mod app;
mod importer;

enum GuiAction {
    Undo,
    Redo,
    SelectNode { node: NodeId },
    AddNode { parent: NodeId, index: u64 },
    MoveNode { node: NodeId, new_parent: NodeId, index_in_new_parent: u64 },
    RemoveNode { node: NodeId },
    WrappedChange { change: Change },
    SetNodeExpanded { node: NodeId, expanded: bool },
    ToggleSelectedNodeExpanded,
    /// Commit pending changes
    Commit,
    SelectPrevious,
    SelectNext,
    SelectParent,
    SelectFirstChild,
    ToggleEditing,
}

enum DragDropPayload {
    NodeId(NodeId),
}

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Explorer", options, move |ctx, _frame| {
        let mut actions: Vec<GuiAction> = vec![];

        check_keyboard(ctx, &app, &mut actions);

        let frame = egui::Frame::default().inner_margin(8.0).fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").default_width(200f32).show(ctx, |ui| {
            create_inspector(ui, app.get_selected_node(), &mut actions);
        });
        egui::TopBottomPanel::bottom("history_panel").default_height(160f32).resizable(true).show(ctx, |ui| {
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

fn check_keyboard(ctx: &Context, app: &SimpleApplication, actions: &mut Vec<GuiAction>) {
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

    if !app.ui.is_editing {
        if app.ui.selected_node.exists() {
            let node = app.ui.selected_node;
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if app.is_node_expanded(node) {
                    actions.push(GuiAction::SetNodeExpanded { node, expanded: false });
                } else {
                    actions.push(GuiAction::SelectParent);
                }
                actions.push(GuiAction::SetNodeExpanded { node, expanded: false });
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                actions.push(GuiAction::SetNodeExpanded { node, expanded: true });
            }
        }
    }
    else {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            actions.push(GuiAction::ToggleEditing);
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
                GuiAction::SelectNode { node } => app.select_node(node),
                GuiAction::AddNode { parent, index } => app.add_child(&parent, index),
                GuiAction::MoveNode { node, new_parent, index_in_new_parent } => app.move_node(&node, &new_parent, index_in_new_parent),
                GuiAction::RemoveNode { node } => app.remove_node(&node),
                GuiAction::Commit => app.commit(),
                GuiAction::WrappedChange { change } => app.document.add_and_apply_change(change),
                GuiAction::Undo => app.document.undo(),
                GuiAction::Redo => app.document.redo(),
                GuiAction::SelectPrevious => app.select_previous(),
                GuiAction::SelectNext => app.select_next(),
                GuiAction::SelectParent => app.select_parent(),
                GuiAction::SelectFirstChild => app.select_first_child(),
                GuiAction::SetNodeExpanded { node, expanded } => app.set_node_expanded(node, expanded),
                GuiAction::ToggleSelectedNodeExpanded => app.toggle_selected_node_expanded(),
                GuiAction::ToggleEditing => app.toggle_editing(),
            }
        }
        None => {}
    }
}

fn create_inspector(ui: &mut Ui, node: Option<&Node>, actions: &mut Vec<GuiAction>) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            egui::Grid::new("inspector_grid").num_columns(2).show(ui, |ui| {
                ui.label("Inspector");
                if ui.button("Delete Node").clicked() {
                    actions.push(GuiAction::RemoveNode { node: node.id });
                }
                ui.end_row();

                let mut name = node.name.clone().unwrap_or_default();
                ui.label("name");
                if ui.text_edit_singleline(&mut name).changed() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetName { node: node.id, name: name.clone() } });
                }
                ui.end_row();

                let mut type_name = node.type_name.clone().unwrap_or_default();
                ui.label("type");
                if ui.text_edit_singleline(& mut type_name).changed() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetType { node: node.id, type_name: type_name.clone() }});
                }
                ui.end_row();

                ui.label("ID");
                ui.label(node.id.to_string());
                ui.end_row();

                for at in node.attributes.iter() {
                    ui.label(&at.key);
                    ui.label(format!("{}", at.value));
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
    ui.collapsing("Pending changes", |ui| {
        if ui.button("Snapshot").clicked() {
            actions.push(GuiAction::Commit);
        }
        for change in &pending.changes {
            ui.label(change.to_string());
        }
    });
    ui.allocate_space(ui.available_size());
}

fn create_tree(ui: &mut Ui, app: &mut SimpleApplication, actions: &mut Vec<GuiAction>) {
    //create_node_tree(ui, app.ui.root, app, actions, 0);
    create_node_tree_children(app, app.ui.root, actions, ui);
}

fn create_node_tree(ui: &mut Ui, node_id: NodeId, app: &SimpleApplication, actions: &mut Vec<GuiAction>, index_in_parent: usize) {
    if let Some(node) = app.document.nodes.get(node_id) {
        let label = get_label(node, index_in_parent, node_id);
        let selected = app.ui.selected_node == node_id;
        let mut text = RichText::new(label);
        if selected {
            text = text.color(ui.visuals().strong_text_color());
        }
        let is_expanded = app.is_node_expanded(node_id);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.dnd_drag_source(Id::new(node_id), node_id, |ui| {
                    ui.label("☰").on_hover_text("Drag to move");
                });

                let expand_icon = if is_expanded { "⏷" } else { "⏵" };
                if ui.label(expand_icon).on_hover_text("Expand/collapse node").clicked() {
                    actions.push(GuiAction::SetNodeExpanded { node: node_id, expanded: !is_expanded });
                }

                let mut checked = node.get_bool_attribute("completed").unwrap_or(false);
                if ui.checkbox(&mut checked, "").clicked() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetBool { node: node_id, attribute: "completed".to_string(), value: checked } });
                }

                if selected && app.ui.is_editing {
                    let mut node_name = node.name.clone().unwrap_or_default();
                    let text_edit = ui.text_edit_singleline(&mut node_name);
                    text_edit.request_focus();
                    if text_edit.changed() {
                        actions.push(GuiAction::WrappedChange { change: Change::SetName { node: node_id, name: node_name.clone() } });
                    }
                } else {
                    let label = ui.label(text);
                    if label.clicked() { actions.push(GuiAction::SelectNode { node: node_id }); }
                    if label.double_clicked() { actions.push(GuiAction::ToggleEditing); }
                }

                ui.spacing();

                if selected {
                    if ui.label("✖").clicked() { actions.push(GuiAction::RemoveNode { node: node_id }); }
                }
            });

            if is_expanded {
                ui.indent(node_id, |ui| {
                    create_node_tree_children(app, node_id, actions, ui);
                });
            }
        });
    }
}

fn get_label(node: &Node, index_in_parent: usize, node_id: NodeId) -> String {
    let name = node.get_name();
    let type_name = node.get_type();

    if let Some(name) = name {
        if let Some(t) = type_name {
            return format!("{}: [{}] {}", index_in_parent, t, name);
        }
        else
        {
            return format!("{}: {}", index_in_parent, name);
        }
    }

    if let Some(t) = type_name {
        return format!("{}: [{}]", index_in_parent, t);
    }

    format!("{}: ID{}", index_in_parent, node_id.index())
}

fn create_node_tree_children(app: &SimpleApplication, node_id: NodeId, actions: &mut Vec<GuiAction>, ui: &mut Ui) {
    let mut index: usize = 0;
    let children =  &app.document.nodes.get(node_id).expect("").children;
    
    for child_id in children {
        create_node_tree(ui, *child_id, app, actions, index);
        index += 1;
    }
    let frame = Frame::default().inner_margin(2.0);

    let (_, dropped_payload) = ui.dnd_drop_zone::<NodeId, ()>(frame, |ui| {
        let add_button = ui.label("⊞").on_hover_text("Add child node");
        if add_button.clicked() {
            actions.push(GuiAction::AddNode { parent: node_id, index: children.len() as u64 });
        }
    });

    if let Some(dropped_id) = dropped_payload {
        actions.push(GuiAction::MoveNode { node: *dropped_id, new_parent: node_id, index_in_new_parent: children.len() as u64 });
    }
}