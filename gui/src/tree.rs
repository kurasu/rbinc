use crate::app::{Application, GuiAction};
use binc::change::Change;
use binc::document::Document;
use binc::node_id::NodeId;
use binc::node_store::Node;
use eframe::egui::StrokeKind::Inside;
use eframe::egui::{Color32, CursorIcon, DragAndDrop, Frame, Id, InnerResponse, LayerId, Order, PointerButton, RichText, Sense, Ui, UiBuilder};
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
        let id2 = ui.make_persistent_id(node_id.index() + 1923490234);


        let header = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    self.dnd_area(
                        ui,
                        app,
                        node,
                        index_in_parent,
                        DragDropPayload::WithNode(node_id),
                        on_action,
                        |ui| {
                            Self::node_frame(ui, app.ui.selected_node == node_id).show(ui, |ui| {
                                ui.label("○");
                                ui.label(node_name);
                            });
                        },
                    );

                    let response = ui.interact(ui.min_rect(), id2, egui::Sense::click());
                    if response.clicked() {
                        on_action(GuiAction::SelectNode { node: node_id });
                    }

                    response.context_menu(|ui| {
                        Self::context_menu(app, node, on_action, node_id, ui);
                    });
                });
            });

        if !node.children.is_empty() {
            header.body(|ui| {
                let children = &app.document.nodes.get(node_id).expect("").children;
                for (index, child_id) in children.iter().enumerate() {
                    self.create_node(ui, app, *child_id, index, on_action);
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

        if is_self_being_dragged && ui.ctx().input(|i| i.pointer.is_decidedly_dragging()) {
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

            // Check for drags:
            let dnd_response = ui
                .interact(response.rect.clone(), id, Sense::drag())
                .on_hover_cursor(CursorIcon::Grab);

            InnerResponse::new(inner, dnd_response | response)
        }
    }

    fn context_menu(app: &Application, node: &Node, on_action: &mut impl FnMut(GuiAction), node_id: NodeId, ui: &mut Ui) {
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
}
