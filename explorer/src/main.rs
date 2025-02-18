#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use std::any::Any;
use eframe::{egui, emath};
use eframe::egui::{Color32, Context, CursorIcon, DragAndDrop, Frame, Id, InnerResponse, LayerId, Order, RichText, Sense, Ui, UiBuilder};
use binc::change::Change;
use binc::node_id::NodeId;
use binc::node_store::Node;
use crate::app::{create_toolbar, Application};

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
    WithNode(NodeId),
}

fn main() -> eframe::Result {
    let mut app = Application::new();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_simple_native("BINC Explorer", options, move |ctx, _frame| {
        let mut actions: Vec<GuiAction> = vec![];

        let mut on_action = |a| {
            actions.push(a);
        };

        check_keyboard(ctx, &app, &mut on_action);

        let frame = egui::Frame::default().inner_margin(8.0).fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
            create_toolbar(&mut app, ui);
        });
        egui::SidePanel::right("inspector_panel").default_width(200f32).show(ctx, |ui| {
            create_inspector(ui, app.get_selected_node(), &mut on_action);
        });
        egui::TopBottomPanel::bottom("history_panel").default_height(160f32).resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                create_history(ui, &app, &mut on_action);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                create_tree(ui, &mut app, &mut on_action);
            });
        });

        for action in actions {
            process_action(Some(action), &mut app);
        }
    })
}

fn check_keyboard(ctx: &Context, app: &Application, on_action: &mut impl FnMut(GuiAction)) {
    if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
        on_action(GuiAction::Undo);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.command) {
        on_action(GuiAction::Redo);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        on_action(GuiAction::SelectPrevious);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        on_action(GuiAction::SelectNext);
    }

    if !app.ui.is_editing {
        if app.ui.selected_node.exists() {
            let node = app.ui.selected_node;
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if app.is_node_expanded(node) {
                    on_action(GuiAction::SetNodeExpanded { node, expanded: false });
                } else {
                    on_action(GuiAction::SelectParent);
                }
                on_action(GuiAction::SetNodeExpanded { node, expanded: false });
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                on_action(GuiAction::SetNodeExpanded { node, expanded: true });
            }
        }
    }
    else {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            on_action(GuiAction::ToggleEditing);
        }
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        on_action(GuiAction::ToggleEditing);
    }
}

fn process_action(action: Option<GuiAction>, app: &mut Application) {
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

fn create_inspector(ui: &mut Ui, node: Option<&Node>, on_action: &mut impl FnMut(GuiAction)) {
    ui.vertical(|ui| {
        if let Some(node) = node {
            egui::Grid::new("inspector_grid").num_columns(2).show(ui, |ui| {
                ui.label("Inspector");
                if ui.button("Delete Node").clicked() {
                    on_action(GuiAction::RemoveNode { node: node.id });
                }
                ui.end_row();

                let mut name = node.name.clone().unwrap_or_default();
                ui.label("name");
                if ui.text_edit_singleline(&mut name).changed() {
                    on_action(GuiAction::WrappedChange { change: Change::SetName { node: node.id, name: name.clone() } });
                }
                ui.end_row();

                let mut type_name = node.type_name.clone().unwrap_or_default();
                ui.label("type");
                if ui.text_edit_singleline(& mut type_name).changed() {
                    on_action(GuiAction::WrappedChange { change: Change::SetType { node: node.id, type_name: type_name.clone() }});
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

fn create_history(ui: &mut Ui, app: &Application, on_action: &mut impl FnMut(GuiAction)) {
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
            on_action(GuiAction::Commit);
        }
        for change in &pending.changes {
            ui.label(change.to_string());
        }
    });
    ui.allocate_space(ui.available_size());
}

fn create_tree(ui: &mut Ui, app: &mut Application, on_action: &mut impl FnMut(GuiAction)) {
    //create_node_tree(ui, app.ui.root, app, actions, 0);
    create_node_tree_children(app, app.ui.root, on_action, ui);
}

fn create_node_tree(ui: &mut Ui, node_id: NodeId, app: &Application, on_action: &mut impl FnMut(GuiAction), index_in_parent: usize) {
    if let Some(node) = app.document.nodes.get(node_id) {
        let label = get_label(node, index_in_parent);
        let selected = app.ui.selected_node == node_id;
        let mut text = RichText::new(label);
        if selected {
            text = text.color(ui.visuals().strong_text_color());
        }
        let is_expanded = app.is_node_expanded(node_id);

        ui.vertical(|ui| {
            expandable_node_header(ui, node, is_expanded, selected, index_in_parent, on_action);

            if is_expanded {
                ui.indent(node_id, |ui| {
                    create_node_tree_children(app, node_id, on_action, ui);
                });
            }
        });
    }
}

fn expandable_node_header(
    ui: &mut Ui,
    node: &Node,
    is_expanded: bool,
    selected: bool,
    index_in_parent: usize,
    on_action: &mut impl FnMut(GuiAction),
) {
    let node_name = get_label(node, index_in_parent);
    let node_id = node.id;
    let mut gui_action : Option<GuiAction> = None;

    dnd_area(ui, node_id, index_in_parent, DragDropPayload::WithNode(node_id),
             |action| gui_action = Some(action),
             |ui| {
        ui.horizontal(|ui| {
            let mut label_color = ui.visuals().text_color();
            if !is_hovering(ui, node_id) {   
                label_color = label_color.linear_multiply(0.04)
            };
            ui.colored_label(label_color, "☰").on_hover_text("Drag to move");

            let expand_icon = if is_expanded { "⏷" } else { "⏵" };
            if ui.label(expand_icon).on_hover_cursor(CursorIcon::Default).on_hover_text("Expand/collapse node").clicked() {
                on_action(GuiAction::SetNodeExpanded { node: node_id, expanded: !is_expanded });
            }

            let mut checked = false; // Replace with actual logic to get the checked state
            if ui.checkbox(&mut checked, "").clicked() {
                on_action(GuiAction::WrappedChange { change: Change::SetBool { node: node_id, attribute: "completed".to_string(), value: checked } });
            }

            if selected {
                let mut node_name = node_name.to_string();
                let text_edit = ui.text_edit_singleline(&mut node_name);
                text_edit.request_focus();
                if text_edit.changed() {
                    on_action(GuiAction::WrappedChange { change: Change::SetName { node: node_id, name: node_name.clone() } });
                }
            } else {
                let label = ui.label(node_name);
                if label.clicked() { on_action(GuiAction::SelectNode { node: node_id }); }
                if label.double_clicked() { on_action(GuiAction::ToggleEditing); }
            }

            ui.spacing();

            if selected {
                if ui.label("✖").clicked() { on_action(GuiAction::RemoveNode { node: node_id }); }
            }
        });
    });
    
    if let Some(action) = gui_action {
        on_action(action);
    }
}

fn is_hovering(ui: &Ui, node_id: NodeId) -> bool {
    if let Some(r) = ui.ctx().read_response(Id::new(node_id)) {
        r.hovered()
    } else {
        false
    }
}

pub fn dnd_area<R>(
    ui: &mut Ui,
    node_id: NodeId,
    index_in_parent: usize,
    payload: DragDropPayload,
    on_action: impl FnOnce(GuiAction) -> (),
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R>
{
    let id= Id::new(node_id);
    let is_self_being_dragged = ui.ctx().is_being_dragged(id);

    if is_self_being_dragged {
        DragAndDrop::set_payload(ui.ctx(), payload);

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
        let InnerResponse { inner, response } =
            ui.scope_builder(UiBuilder::new().layer_id(layer_id), add_contents);

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let delta = pointer_pos - response.rect.left_center();
            ui.ctx()
                .transform_layer_shapes(layer_id, emath::TSTransform::from_translation(delta));
        }

        InnerResponse::new(inner, response)
    } else {
        let InnerResponse { inner, response } = ui.scope(add_contents);

        if let (Some(pointer), Some(hovered_payload)) = (
            ui.input(|i| i.pointer.interact_pos()),
            response.dnd_hover_payload::<DragDropPayload>(),
        ) {
            let rect = response.rect;

            // Preview insertion:
            let stroke = egui::Stroke::new(1.0, Color32::WHITE);
            let insert_idx = if pointer.y < rect.center().y {
                // Above us
                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                index_in_parent
            } else {
                // Below us
                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                index_in_parent + 1
            };

            if let Some(dragged_payload) = response.dnd_release_payload::<DragDropPayload>() {
                // The user dropped onto this item.
                let d = dragged_payload.as_ref();
                let action = match d {
                    DragDropPayload::WithNode(node) => {
                        on_action(GuiAction::MoveNode { node: *node, new_parent: node_id, index_in_new_parent: insert_idx as u64 });
                    }
                };
            }
        }

        // Check for drags:
        let mut small_rect = response.rect.clone();
        small_rect.set_width(20f32);
        let dnd_response = ui
            .interact(small_rect, id, Sense::drag())
            .on_hover_cursor(CursorIcon::Grab);

        InnerResponse::new(inner, dnd_response | response)
    }
}

fn get_label(node: &Node, index_in_parent: usize) -> String {
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

    format!("{}: ID{}", index_in_parent, node.id.index())
}

fn create_node_tree_children(app: &Application, node_id: NodeId, on_action: &mut impl FnMut(GuiAction), ui: &mut Ui) {
    let mut index: usize = 0;
    let children =  &app.document.nodes.get(node_id).expect("").children;
    
    for child_id in children {
        create_node_tree(ui, *child_id, app, on_action, index);
        index += 1;
    }
    let frame = Frame::default().inner_margin(2.0);

    let (_, dropped_payload) = ui.dnd_drop_zone::<NodeId, ()>(frame, |ui| {
        let add_button = ui.label("⊞").on_hover_text("Add child node");
        if add_button.clicked() {
            on_action(GuiAction::AddNode { parent: node_id, index: children.len() as u64 });
        }
    });

    if let Some(dropped_id) = dropped_payload {
        on_action(GuiAction::MoveNode { node: *dropped_id, new_parent: node_id, index_in_new_parent: children.len() as u64 });
    }
}