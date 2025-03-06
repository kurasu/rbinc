#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use binc::node_id::NodeId;
use binc::node_store::Node;
use binc::operation::Operation;
use bincgui::app::{create_toolbar, Application, GuiAction};
use bincgui::column::Columns;
use bincgui::history::History;
use bincgui::tree::NodeTree;
use eframe::egui::{Context, Ui};
use eframe::{egui, Frame};

mod notes;

struct ExplorerApp {
    application: Application,
    history: History,
    tree: NodeTree,
    columns: Columns,
    use_tree: bool,
}

impl ExplorerApp {
    fn new() -> Self {
        Self {
            application: Application::new(),
            history: History::new(),
            tree: NodeTree::new(),
            columns: Columns::new(),
            use_tree: true,
        }
    }

    fn get_node(&self, id: NodeId) -> &Node {
        self.application.get(id).expect("Node not found")
    }

    fn node_type(&self, id: NodeId) -> String {
        match self.get_node(id).type_id {
            Some(id) => self.application.document.type_name(Some(id)),
            None => format!("Type #{}", id),
        }
    }

    fn check_keyboard(
        ctx: &Context,
        app: &Application,
        use_tree: bool,
        on_action: &mut impl FnMut(GuiAction),
    ) {
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
            on_action(GuiAction::Undo);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.command) {
            on_action(GuiAction::Redo);
        }
        if use_tree {
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                on_action(GuiAction::SelectPreviousInTree);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                on_action(GuiAction::SelectNextInTree);
            }

            if !app.ui.is_editing {
                if app.ui.selected_node.exists() {
                    let node = app.ui.selected_node;
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                        if app.is_node_expanded(node) {
                            on_action(GuiAction::SetNodeExpanded {
                                node,
                                expanded: false,
                            });
                        } else {
                            on_action(GuiAction::SelectParent);
                        }
                        on_action(GuiAction::SetNodeExpanded {
                            node,
                            expanded: false,
                        });
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                        on_action(GuiAction::SetNodeExpanded {
                            node,
                            expanded: true,
                        });
                    }
                }
            }
        } else {
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                on_action(GuiAction::SelectPreviousSibling);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                on_action(GuiAction::SelectNextSibling);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                on_action(GuiAction::SelectParent);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                on_action(GuiAction::SelectFirstChild);
            }
        }

        if app.ui.is_editing {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                on_action(GuiAction::ToggleEditing);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            on_action(GuiAction::ToggleEditing);
        }
    }

    fn create_inspector(&mut self, ui: &mut Ui, on_action: &mut impl FnMut(GuiAction)) {
        ui.vertical(|ui| {
            if let Some(node) = self.application.get_selected_node() {
                egui::Grid::new("inspector_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Inspector");
                        if ui.button("Delete Node").clicked() {
                            on_action(GuiAction::RemoveNode { node: node.id });
                        }
                        ui.end_row();

                        let mut name = node.name.clone().unwrap_or_default();
                        ui.label("name");
                        if ui.text_edit_singleline(&mut name).changed() {
                            on_action(GuiAction::WrappedChange {
                                change: Operation::SetName {
                                    node: node.id,
                                    name: name.clone(),
                                },
                            });
                        }
                        ui.end_row();

                        ui.label("type");
                        /*if ui.text_edit_singleline(&mut type_name).changed() {
                            on_action(GuiAction::WrappedChange {
                                change: Change::SetType {
                                    node: node.id,
                                    type_name: type_name.clone(),
                                },
                            });
                        }*/
                        ui.label(self.node_type(node.id));
                        ui.end_row();

                        ui.label("ID");
                        ui.label(node.id.to_string());
                        ui.end_row();

                        for at in node.attributes.iter() {
                            ui.label(self.application.document.attribute_name(at.key));
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
}

impl eframe::App for ExplorerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut actions: Vec<GuiAction> = vec![];

        let mut on_action = |a| {
            actions.push(a);
        };

        Self::check_keyboard(ctx, &mut self.application, self.use_tree, &mut on_action);

        let frame = egui::Frame::default()
            .inner_margin(8.0)
            .fill(ctx.style().visuals.panel_fill);
        egui::TopBottomPanel::top("toolbar")
            .frame(frame)
            .show(ctx, |ui| {
                create_toolbar(&mut self.application, ui, |ui| {
                    ui.checkbox(&mut self.history.show_history, "Show History");
                });
            });
        egui::SidePanel::right("inspector_panel")
            .default_width(200f32)
            .show(ctx, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    self.create_inspector(ui, &mut on_action);
                });
            });

        if self.history.show_history {
            egui::TopBottomPanel::bottom("history_panel")
                .default_height(160f32)
                .resizable(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            self.history.create_history(
                                ui,
                                &self.application.document.repository,
                                self.application.document.undo_revision,
                                &mut on_action,
                            );
                        });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().style_mut(|style| {
                style.interaction.selectable_labels = false;
            });
            if self.use_tree {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        self.tree
                            .create_tree(ui, &mut self.application, &mut on_action);
                    });
            } else {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    self.columns
                        .create_columns(ui, &mut self.application, &mut on_action);
                });
            }
        });

        for action in actions {
            self.application.process_action(action);
        }
    }
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "BINC Explorer",
        options,
        Box::new(|_cc| Ok(Box::new(ExplorerApp::new()))),
    )
}
