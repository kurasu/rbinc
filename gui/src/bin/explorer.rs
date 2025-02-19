#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]

use binc::change::Change;
use binc::node_store::Node;
use bincgui::app::{create_toolbar, Application, GuiAction};
use bincgui::history::create_history;
use eframe::egui::{Context, Ui};
use eframe::egui;
use std::any::Any;
use bincgui::tree::create_tree;

mod notes;


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
            app.process_action(action);
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