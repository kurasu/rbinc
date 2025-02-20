use crate::app::{Application, GuiAction};
use binc::attributes::AttributeValue;
use binc::change::Change;
use binc::node_id::NodeId;
use binc::node_store::Node;
use eframe::egui::StrokeKind::Inside;
use eframe::egui::{
    CursorIcon, DragAndDrop, Frame, Id, InnerResponse, LayerId, Order, Sense, Ui, UiBuilder,
};
use eframe::{egui, emath};
use egui_extras::{Column, TableBuilder};

enum DragDropPayload {
    WithNode(NodeId),
}

pub struct Columns {
    root_node: NodeId,
    expanded_columns: Vec<NodeId>,
}

impl Columns {
    pub fn new() -> Self {
        Columns {
            root_node: NodeId::ROOT_NODE,
            expanded_columns: vec![],
        }
    }

    pub fn create_columns(
        &mut self,
        ui: &mut Ui,
        app: &mut Application,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        self.expanded_columns = Self::get_columns_to_show(app);

        ui.horizontal(|ui| {
            for node in &self.expanded_columns {
                self.create_column(app, node.clone(), on_action, ui);
            }
        });
    }

    fn get_columns_to_show(app: &Application) -> Vec<NodeId> {
        let mut node_id = app.ui.selected_node;

        if node_id == NodeId::NO_NODE {
            return vec![NodeId::ROOT_NODE];
        }

        let mut columns = vec![];
        while node_id != NodeId::NO_NODE {
            let node = app.get(node_id).expect("Node must exists");
            if !node.children.is_empty() || !columns.is_empty() {
                columns.push(node_id);
            }
            node_id = node.parent;
        }
        columns.reverse();
        columns
    }

    fn create_column(
        &self,
        app: &Application,
        node_id: NodeId,
        on_action: &mut impl FnMut(GuiAction),
        ui: &mut Ui,
    ) {
        let frame = Frame::default().inner_margin(2.0);
        let node = app.document.nodes.get(node_id).expect("");

        ui.push_id(node_id, |ui| {
            ui.vertical(|ui| {
                //let (_, dropped_payload) = ui.dnd_drop_zone::<NodeId, ()>(frame, |ui| {
                let available_height = ui.available_height();
                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::exact(150.0))
                    .min_scrolled_height(available_height)
                    .max_scroll_height(available_height);

                table = table.sense(Sense::click());

                let mut index: usize = 0;
                let children = &app.document.nodes.get(node_id).expect("").children;

                table
                    /*.header(22.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Name");
                        });
                    })*/
                    .body(|mut body| {
                        for child_id in children {
                            body.row(22.0, |mut row| {
                                row.set_selected(app.ui.selected_node == *child_id);
                                row.col(|ui| {
                                    ui.set_width(ui.available_width());
                                    self.create_node_header(ui, app, *child_id, index, on_action);
                                });

                                if row.response().clicked() {
                                    on_action(GuiAction::SelectNode { node: *child_id });
                                }
                            });
                            index += 1;
                        }
                    });

                let add_button = ui.label("⊞").on_hover_text("Add child node");
                if add_button.clicked() {
                    on_action(GuiAction::AddNode {
                        parent: node_id,
                        index: children.len() as u64,
                    });
                }
            });
        });
        //});

        /*if let Some(dropped_id) = dropped_payload {
            on_action(GuiAction::MoveNode {
                node: *dropped_id,
                new_parent: node_id,
                index_in_new_parent: node.children.len() as u64,
            });
        }*/
    }

    fn create_node_header(
        &self,
        ui: &mut Ui,
        app: &Application,
        node_id: NodeId,
        index_in_parent: usize,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        let node = app.document.nodes.get(node_id).expect("");
        let node_name = self.get_label(node, index_in_parent);
        let node_id = node.id;
        let mut gui_action: Option<GuiAction> = None;

        self.dnd_area(
            ui,
            app,
            node,
            index_in_parent,
            DragDropPayload::WithNode(node_id),
            |action| gui_action = Some(action),
            |ui| {
                ui.horizontal(|ui| {
                    if let Some(mut checked) = node.get_bool_attribute("completed") {
                        if ui.checkbox(&mut checked, "").clicked() {
                            on_action(GuiAction::WrappedChange {
                                change: Change::SetAttribute {
                                    node: node_id,
                                    attribute: "completed".to_string(),
                                    value: AttributeValue::Bool(checked),
                                },
                            });
                        }
                    }

                    let selected = app.ui.selected_node == node_id;

                    if selected && app.ui.is_editing {
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
                        let label = ui.label(node_name);
                        if label.clicked() {
                            on_action(GuiAction::SelectNode { node: node_id });
                        }
                        if label.double_clicked() {
                            on_action(GuiAction::ToggleEditing);
                        }
                    }

                    let space = ui.available_size() - emath::vec2(20.0, 0.0);
                    if space.x > 0.0 {
                        ui.allocate_space(space);
                    }

                    if !node.children.is_empty() {
                        ui.label("▶");
                    }
                });
            },
        );

        ui.response().context_menu(|ui| {
            if ui.button("Add child node").clicked() {
                on_action(GuiAction::AddNode {
                    parent: node_id,
                    index: node.children.len() as u64,
                });
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
        on_action: impl FnOnce(GuiAction) -> (),
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
                                    index_in_new_parent: insert_idx as u64,
                                });
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

    fn get_label(&self, node: &Node, index_in_parent: usize) -> String {
        let name = node.get_name();
        let type_name = node.get_type();

        if let Some(name) = name {
            if let Some(t) = type_name {
                return format!("{}: [{}] {}", index_in_parent, t, name);
            } else {
                return format!("{}: {}", index_in_parent, name);
            }
        }

        if let Some(t) = type_name {
            return format!("{}: [{}]", index_in_parent, t);
        }

        format!("{}: ID{}", index_in_parent, node.id.index())
    }
}
