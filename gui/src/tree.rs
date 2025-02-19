use eframe::egui::{CursorIcon, DragAndDrop, Frame, Id, InnerResponse, LayerId, Order, RichText, Sense, Ui, UiBuilder};
use eframe::{egui, emath};
use eframe::egui::StrokeKind::Inside;
use binc::attributes::AttributeValue;
use binc::change::Change;
use binc::node_id::NodeId;
use binc::node_store::Node;
use crate::app::{Application, GuiAction};

pub fn create_tree(ui: &mut Ui, app: &mut Application, on_action: &mut impl FnMut(GuiAction)) {
    create_node_tree_children(app, app.ui.root, on_action, ui);
}

enum DragDropPayload {
    WithNode(NodeId),
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
            expandable_node_header(ui, app, node, is_expanded, selected, index_in_parent, on_action);

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
    app: &Application,
    node: &Node,
    is_expanded: bool,
    selected: bool,
    index_in_parent: usize,
    on_action: &mut impl FnMut(GuiAction),
) {
    let node_name = get_label(node, index_in_parent);
    let node_id = node.id;
    let mut gui_action : Option<GuiAction> = None;

    dnd_area(ui, app, node, index_in_parent, DragDropPayload::WithNode(node_id),
             |action| gui_action = Some(action),
             |ui| {
                 ui.horizontal(|ui| {
                     let mut label_color = ui.visuals().text_color();
                     if !is_hovering(ui, node_id) {
                         label_color = ui.visuals().weak_text_color();
                     };
                     ui.colored_label(label_color, "☰").on_hover_text("Drag to move");

                     let expand_icon = if is_expanded { "⏷" } else { "⏵" };
                     if ui.label(expand_icon).on_hover_cursor(CursorIcon::Default).on_hover_text("Expand/collapse node").clicked() {
                         on_action(GuiAction::SetNodeExpanded { node: node_id, expanded: !is_expanded });
                     }

                     let mut checked = node.get_bool_attribute("completed").unwrap_or_default();
                     if ui.checkbox(&mut checked, "").clicked() {
                         on_action(GuiAction::WrappedChange { change: Change::SetAttribute { node: node_id, attribute: "completed".to_string(), value: AttributeValue::Bool(checked) } });
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

    ui.response().context_menu(|ui|  {
        if ui.button("Add child node").clicked() {
            on_action(GuiAction::AddNode { parent: node_id, index: node.children.len() as u64 });
            ui.close_menu()
        }
        if ui.button("Delete").clicked() {
            on_action(GuiAction::RemoveNode { node: node_id });
            ui.close_menu()
        }
        for tag in node.tags.iter() {
            ui.label(tag);
        }
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
    app: &Application,
    node: &Node,
    index_in_parent: usize,
    payload: DragDropPayload,
    on_action: impl FnOnce(GuiAction) -> (),
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R>
{
    let node_id = node.id;
    let parent_id = node.parent;
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
            let edge_margin = 4f32;
            let y1 = rect.top() + edge_margin;
            let y2 = rect.bottom() - edge_margin;

            let allowed = match hovered_payload.as_ref() {
                DragDropPayload::WithNode(hovered_node_id) => {
                    // Don't allow dropping onto self or children of self:
                    *hovered_node_id != node_id
                        && !is_ancestor(app, node_id, *hovered_node_id)
                }
            };

            // Preview insertion:
            let visuals = &ui.ctx().style().visuals;
            let color = if allowed { visuals.text_color() } else { visuals.error_fg_color };
            let stroke = egui::Stroke::new(1.0, color);
            let (target, insert_idx) = if pointer.y < y1 {
                // Above us
                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                (parent_id, index_in_parent)
            } else if pointer.y < y2 {
                // On us
                ui.painter().rect_stroke(rect, 2, stroke, Inside);
                (node_id, node.children.len())
            } else {
                // Below us
                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                (parent_id, index_in_parent + 1)
            };

            if allowed {
                if let Some(dragged_payload) = response.dnd_release_payload::<DragDropPayload>() {
                    // The user dropped onto this item.
                    let d = dragged_payload.as_ref();
                    match d {
                        DragDropPayload::WithNode(node) => {
                            on_action(GuiAction::MoveNode { node: *node, new_parent: target, index_in_new_parent: insert_idx as u64 });
                        }
                    };
                }
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

fn is_ancestor(app: &Application, node_id: NodeId, assumed_ancestor: NodeId) -> bool {
    let node = app.document.nodes.get(node_id).expect("");

    if node.parent == assumed_ancestor {
        return true;
    }
    if node.parent == NodeId::NO_NODE {
        return false;
    }

    is_ancestor(app, node.parent, assumed_ancestor)
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