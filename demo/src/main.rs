#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use eframe::egui::{Context, RichText, Ui};
use binc::change::Change;
use binc::node_id::NodeId;
use binc::node_store::Node;
use gui::gui::*;

enum GuiAction {
    Undo,
    Redo,
    SelectNode { node: NodeId },
    AddNode { parent: NodeId, index: u64 },
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

fn main() -> eframe::Result {
    let mut app = SimpleApplication::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    let mut name = "hello".to_string(); //&mut app.selected_node_name;
    

    eframe::run_simple_native("BINC Demo", options, move |ctx, _frame| {
        let mut actions: Vec<GuiAction> = vec![];

        check_keyboard(ctx, &app, &mut actions);

        let frame = egui::Frame::default().inner_margin(8.0).fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").default_width(200f32).show(ctx, |ui| {
            create_inspector(ui, app.get_selected_node(), &mut name, &mut actions);
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
                if app.ui.expanded_nodes.contains(&node) {
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

fn create_inspector(ui: &mut Ui, node: Option<&Node>, node_name: &mut String, actions: &mut Vec<GuiAction>) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            egui::Grid::new("inspector_grid").num_columns(2).show(ui, |ui| {
                ui.label("Inspector");
                if ui.button("Delete Node").clicked() {
                    actions.push(GuiAction::RemoveNode { node: node.id });
                }
                ui.end_row();

                ui.label("name");
                if ui.text_edit_singleline(node_name).changed() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetString { node: node.id, attribute: "name".to_string(), value: node_name.clone() } });
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
        let id_string = format!("{}: ID{}", index_in_parent, node_id.index());
        let name = node.get_string_attribute("name");
        let mut node_name = name.unwrap_or(String::new()).clone();
        let label = get_label(id_string, &node_name);
        let selected = app.ui.selected_node == node_id;
        let mut text = RichText::new(label);
        if selected {
            text = text.color(ui.visuals().strong_text_color());
        }
        let is_expanded = app.ui.expanded_nodes.contains(&node_id);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let expand_icon = if is_expanded { "⏷" } else { "⏵" };
                if ui.label(expand_icon).on_hover_text("Expand/collapse node").clicked() {
                    actions.push(GuiAction::SetNodeExpanded { node: node_id, expanded: !is_expanded });
                }

                let mut checked = node.get_bool_attribute("completed").unwrap_or(false);
                if ui.checkbox(&mut checked, "").clicked() {
                    actions.push(GuiAction::WrappedChange { change: Change::SetBool { node: node_id, attribute: "completed".to_string(), value: checked } });
                }

                if selected && app.ui.is_editing {
                    let text_edit = ui.text_edit_singleline(&mut node_name);
                    text_edit.request_focus();
                    if text_edit.changed() {
                        actions.push(GuiAction::WrappedChange { change: Change::SetString { node: node_id, attribute: "name".to_string(), value: node_name.clone() } });
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

fn create_node_tree_children(app: &SimpleApplication, node_id: NodeId, actions: &mut Vec<GuiAction>, ui: &mut Ui) {
    let mut index: usize = 0;
    let children =  &app.document.nodes.get(node_id).expect("").children;
    
    for child_id in children {
        create_node_tree(ui, *child_id, app, actions, index);
        index += 1;
    }
    let add_button = ui.label("⊞").on_hover_text("Add child node");
    if add_button.clicked() {
        actions.push(GuiAction::AddNode { parent: node_id, index: children.len() as u64 });
    }
}

fn get_label(id_string: String, name: &String) -> String {
    if !name.is_empty() {
        let name = name;
        format!("{}", name)
    } else { id_string }
}