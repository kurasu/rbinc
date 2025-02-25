use crate::app::{Application, GuiAction};
use binc::change::Change;
use binc::document::Document;
use binc::node_id::NodeId;
use binc::node_store::Node;
use eframe::egui::StrokeKind::Inside;
use eframe::egui::{
    Color32, CursorIcon, DragAndDrop, Frame, Id, InnerResponse, LayerId, Order, RichText, Ui,
    UiBuilder,
};
use eframe::{egui, emath};

enum DragDropPayload {
    WithNode(NodeId),
}

pub struct NodeTree {
    root_node: NodeId,
}

impl NodeTree {
    pub fn new() -> Self {
        NodeTree {
            root_node: NodeId::ROOT_NODE,
        }
    }

    pub fn create_tree(
        &self,
        ui: &mut Ui,
        app: &mut Application,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        //self.create_node_tree_children(app, self.root_node, on_action, ui);
        self.create_node(ui, app, self.root_node, 0, on_action)
    }

    fn create_node(
        &self,
        ui: &mut Ui,
        app: &Application,
        node_id: NodeId,
        index_in_parent: usize,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        let node = app.document.nodes.get(node_id).expect("");
        let node_name = self.get_label(&app.document, node, index_in_parent);

        let id = ui.make_persistent_id(node_id);
        let drag_id = ui.make_persistent_id(node_id.index() + 0923490234);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.dnd_drag_source(drag_id, DragDropPayload::WithNode(node_id), |ui| {
                        ui.label("○");
                    });
                    ui.label(node_name)
                });
            })
            .body(|ui| {
                let children = &app.document.nodes.get(node_id).expect("").children;
                for (index, child_id) in children.iter().enumerate() {
                    self.create_node(ui, app, *child_id, index, on_action);
                }
            });
    }

    fn create_node_and_children(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        app: &Application,
        on_action: &mut impl FnMut(GuiAction),
        index_in_parent: usize,
    ) {
        if let Some(node) = app.document.nodes.get(node_id) {
            let selected = app.ui.selected_node == node_id;
            let is_expanded = app.is_node_expanded(node_id);

            ui.vertical(|ui| {
                self.expandable_node_header(
                    ui,
                    app,
                    node,
                    is_expanded,
                    selected,
                    index_in_parent,
                    on_action,
                );

                if is_expanded {
                    ui.indent(node_id, |ui| {
                        self.create_node_tree_children(app, node_id, on_action, ui);
                    });
                }
            });
        }
    }

    fn node_frame(ui: &Ui, selected: bool) -> Frame {
        if selected {
            Frame::default()
                .fill(ui.visuals().selection.bg_fill)
                .corner_radius(4.0)
                .inner_margin(4.0)
        } else {
            Frame::new().inner_margin(4.0)
        }
    }

    fn expandable_node_header(
        &self,
        ui: &mut Ui,
        app: &Application,
        node: &Node,
        is_expanded: bool,
        selected: bool,
        index_in_parent: usize,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        let node_name = self.get_label(&app.document, node, index_in_parent);
        let node_id = node.id;
        let mut gui_action: Option<GuiAction> = None;

        self.dnd_area(
            ui,
            app,
            node,
            index_in_parent,
            DragDropPayload::WithNode(node_id),
            &mut |action| gui_action = Some(action),
            |ui| {
                Self::node_frame(ui, selected).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let has_children = !node.children.is_empty();

                        if has_children {
                            let expand_icon = if is_expanded { "⏷" } else { "⏵" };
                            if ui
                                .label(expand_icon)
                                .on_hover_cursor(CursorIcon::Default)
                                .on_hover_text("Expand/collapse node")
                                .clicked()
                            {
                                on_action(GuiAction::SetNodeExpanded {
                                    node: node_id,
                                    expanded: !is_expanded,
                                });
                            }
                        } else {
                            ui.colored_label(Color32::TRANSPARENT, "⏵");
                        }

                        let can_edit_name = false;

                        if selected && app.ui.is_editing && can_edit_name {
                            let mut node_name = node.name.clone().unwrap_or_default();
                            let text_edit = ui.text_edit_singleline(&mut node_name);
                            text_edit.request_focus();
                            if text_edit.changed() {
                                on_action(GuiAction::WrappedChange {
                                    change: Change::SetName {
                                        node: node_id,
                                        name: node_name.clone(),
                                    },
                                });
                            }
                        } else {
                            let mut text = RichText::new(node_name);
                            if selected {
                                text = text.color(ui.visuals().strong_text_color());
                            }

                            let label = ui.label(text);
                            if label.clicked() {
                                on_action(GuiAction::SelectNode { node: node_id });
                            }
                            if label.double_clicked() {
                                on_action(GuiAction::ToggleEditing);
                            }
                        }

                        if selected && !app.ui.is_editing {
                            if ui.label("✖").clicked() {
                                on_action(GuiAction::RemoveNode { node: node_id });
                            }
                        }
                    });
                });
            },
        );

        ui.response().context_menu(|ui| {
            if ui.button("Add child node").clicked() {
                on_action(GuiAction::AddNode {
                    parent: node_id,
                    index: node.children.len(),
                });
                ui.close_menu()
            }
            if ui.button("Delete").clicked() {
                on_action(GuiAction::RemoveNode { node: node_id });
                ui.close_menu()
            }
            for tag in node.tags.iter() {
                ui.label(app.document.tag_name(*tag));
            }
        });

        if let Some(action) = gui_action {
            on_action(action);
        }
    }

    fn is_hovering(&self, ui: &Ui, node_id: NodeId) -> bool {
        if let Some(r) = ui.ctx().read_response(Id::new(node_id)) {
            r.hovered()
        } else {
            false
        }
    }

    pub fn dnd_area<R>(
        &self,
        ui: &mut Ui,
        app: &Application,
        node: &Node,
        index_in_parent: usize,
        payload: DragDropPayload,
        on_action: &mut impl FnMut(GuiAction) -> (),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let node_id = node.id;
        let parent_id = node.parent;
        let id = Id::new(node_id);
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
                            && !self.is_ancestor(app, node_id, *hovered_node_id)
                    }
                };

                // Preview insertion:
                let visuals = &ui.ctx().style().visuals;
                let color = if allowed {
                    visuals.text_color()
                } else {
                    visuals.error_fg_color
                };
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
                    if let Some(dragged_payload) = response.dnd_release_payload::<DragDropPayload>()
                    {
                        // The user dropped onto this item.
                        let d = dragged_payload.as_ref();
                        match d {
                            DragDropPayload::WithNode(node) => {
                                on_action(GuiAction::MoveNode {
                                    node: *node,
                                    new_parent: target,
                                    index_in_new_parent: insert_idx,
                                });
                            }
                        };
                    }
                }
            }

            if !response.is_pointer_button_down_on() {
                let dnd_response = ui.response();
                if dnd_response.clicked() {
                    on_action(GuiAction::SelectNode { node: node_id });
                } else if dnd_response.drag_started() {
                    ui.ctx().set_dragged_id(id);
                }
            }

            InnerResponse::new(inner, response)
        }
    }

    fn is_ancestor(&self, app: &Application, node_id: NodeId, assumed_ancestor: NodeId) -> bool {
        let node = app.document.nodes.get(node_id).expect("");

        if node.parent == assumed_ancestor {
            return true;
        }
        if node.parent == NodeId::NO_NODE {
            return false;
        }

        self.is_ancestor(app, node.parent, assumed_ancestor)
    }

    fn get_label(&self, document: &Document, node: &Node, index_in_parent: usize) -> String {
        let name = node.get_name();
        let type_id = node.get_type();
        let type_name = document.type_name(type_id);

        if let Some(name) = name {
            if let Some(t) = type_id {
                return format!("{}: [{}] {}", index_in_parent, type_name, name);
            } else {
                return format!("{}: {}", index_in_parent, name);
            }
        }

        if let Some(t) = type_id {
            return format!("{}: [{}]", index_in_parent, type_name);
        }

        format!("{}: ID{}", index_in_parent, node.id.index())
    }

    fn create_node_tree_children(
        &self,
        app: &Application,
        node_id: NodeId,
        on_action: &mut impl FnMut(GuiAction),
        ui: &mut Ui,
    ) {
        let mut index: usize = 0;
        let children = &app.document.nodes.get(node_id).expect("").children;

        for child_id in children {
            //self.create_node_tree(ui, *child_id, app, on_action, index);
            index += 1;
        }
    }
}
