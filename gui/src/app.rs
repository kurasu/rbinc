use crate::importer::{Import, Importer, IMPORTERS};
use binc::change::Change;
use binc::changes::Changes;
use binc::client::Client;
use binc::document::Document;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::node_id::NodeId;
use binc::node_store::Node;
use binc::repository::Repository;
use eframe::egui;
use eframe::egui::{Button, Id, Modal, Sense, Ui, Widget};
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub enum GuiAction {
    Undo,
    Redo,
    SelectNode {
        node: NodeId,
    },
    AddNode {
        parent: NodeId,
        index: u64,
    },
    MoveNode {
        node: NodeId,
        new_parent: NodeId,
        index_in_new_parent: u64,
    },
    RemoveNode {
        node: NodeId,
    },
    WrappedChange {
        change: Change,
    },
    SetNodeExpanded {
        node: NodeId,
        expanded: bool,
    },
    ToggleSelectedNodeExpanded,
    /// Commit pending changes
    Commit {
        message: String,
    },
    SelectPreviousInTree,
    SelectNextInTree,
    SelectPreviousSibling,
    SelectNextSibling,
    SelectParent,
    SelectFirstChild,
    ToggleEditing,
}

#[derive(Debug)]
pub struct UiState {
    pub root: NodeId,
    pub selected_node: NodeId,
    pub selected_node_name: String,
    expanded_nodes: HashSet<NodeId>,
    pub is_editing: bool,
    host_address: String,
    show_connect_dialog: bool,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            root: NodeId::ROOT_NODE,
            selected_node: NodeId::NO_NODE,
            selected_node_name: String::new(),
            expanded_nodes: HashSet::new(),
            is_editing: false,
            host_address: "localhost:2525/test-file.binc".to_string(),
            show_connect_dialog: false,
        }
    }
}

pub struct Application {
    pub document: Box<Document>,
    pub ui: UiState,
    document_path: Option<PathBuf>,
    client: Option<Client>,
}

impl Application {
    pub fn root(&self) -> &Node {
        self.document
            .nodes
            .get(self.ui.root)
            .expect("Root node should exist")
    }

    pub(crate) fn connect_to_url(&mut self, url: &str) {
        let (host, path) = url.split_once('/').unwrap();
        let result = Self::connect_to_document(url);
        if let Ok((client, document)) = result {
            self.client = Some(client);
            self.document = Box::from(document);
        } else if let Err(error) = result {
            let text = format!("Failed to connect to host\n\n{}", error.to_string());
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Error")
                .set_description(text)
                .show();
        }
    }

    fn connect_to_document(url: &str) -> io::Result<(Client, Document)> {
        if let Some((host, path)) = url.split_once('/') {
            if let Ok(mut client) = Client::new(host) {
                if let NetworkResponse::GetFileData {
                    from_revision,
                    to_revision,
                    data,
                } = client.request(NetworkRequest::GetFileData {
                    from_revision: 0,
                    path: path.to_string(),
                })? {
                    let repo = Repository::read(&mut &data[..])?;
                    let document = Document::new(repo);

                    Ok((client, document))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to get file data",
                    ))
                }
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to connect to host",
                ))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid URL"))
        }
    }
}

impl Application {
    pub fn process_action(&mut self, action: GuiAction) {
        match action {
            GuiAction::SelectNode { node } => self.select_node(node),
            GuiAction::AddNode { parent, index } => self.add_child(&parent, index),
            GuiAction::MoveNode {
                node,
                new_parent,
                index_in_new_parent,
            } => self.move_node(&node, &new_parent, index_in_new_parent),
            GuiAction::RemoveNode { node } => self.remove_node(&node),
            GuiAction::Commit { message } => self.commit(&message),
            GuiAction::WrappedChange { change } => self.document.add_and_apply_change(change),
            GuiAction::Undo => self.document.undo(),
            GuiAction::Redo => self.document.redo(),
            GuiAction::SelectPreviousInTree => self.select_previous_in_tree(),
            GuiAction::SelectNextInTree => self.select_next_in_tree(),
            GuiAction::SelectPreviousSibling => self.select_previous_sibling(),
            GuiAction::SelectNextSibling => self.select_next_sibling(),
            GuiAction::SelectParent => self.select_parent(),
            GuiAction::SelectFirstChild => self.select_first_child(),
            GuiAction::SetNodeExpanded { node, expanded } => self.set_node_expanded(node, expanded),
            GuiAction::ToggleSelectedNodeExpanded => self.toggle_selected_node_expanded(),
            GuiAction::ToggleEditing => self.toggle_editing(),
        }
    }
}

impl Application {
    pub fn get_selected_node(&self) -> Option<&Node> {
        if self.ui.selected_node.exists() {
            return self.document.nodes.get(self.ui.selected_node);
        }
        None
    }
}

impl Application {
    pub fn new() -> Application {
        Application {
            document: Box::from(new_document()),
            ui: UiState::default(),
            document_path: None,
            client: None,
        }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Box::from(document);
        self.ui.root = NodeId::ROOT_NODE;
        self.select_node(NodeId::NO_NODE);
    }

    pub fn select_node(&mut self, node_id: NodeId) {
        self.ui.selected_node = node_id;
        if node_id.exists() {
            let name = self
                .document
                .nodes
                .get(node_id)
                .as_ref()
                .expect("Should exist")
                .get_string_attribute("name");
            self.ui.selected_node_name = name.unwrap_or(String::new());
        } else {
            self.ui.selected_node_name = String::new();
        }
    }

    pub fn add_child(&mut self, parent_id: &NodeId, insertion_index: u64) {
        let child_id = self.document.next_id();

        let c1 = Change::AddNode {
            id: child_id,
            parent: parent_id.clone(),
            index_in_parent: insertion_index,
        };
        self.document.add_and_apply_change(c1);
    }

    pub fn move_node(&mut self, node_id: &NodeId, new_parent_id: &NodeId, insertion_index: u64) {
        let c = Change::MoveNode {
            id: node_id.clone(),
            new_parent: new_parent_id.clone(),
            index_in_new_parent: insertion_index,
        };
        self.document.add_and_apply_change(c);
    }

    pub fn remove_node(&mut self, node_id: &NodeId) {
        let c = Change::DeleteNode {
            id: node_id.clone(),
        };
        self.document.add_and_apply_change(c);
        self.select_node(NodeId::NO_NODE);
        if !self.node_exists(self.ui.root) {
            self.ui.root = NodeId::ROOT_NODE;
        }
    }

    fn node_exists(&self, id: NodeId) -> bool {
        self.document.nodes.exists(id)
    }

    pub fn commit(&mut self, message: &str) {
        self.document.pending_changes.message = message.to_string();
        self.document.commit_changes();
    }

    pub fn get_previous_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(node_id) {
            let parent = node.parent;
            if parent.exists() {
                let children = self
                    .document
                    .nodes
                    .get(parent)
                    .as_ref()
                    .expect("Should exist")
                    .children
                    .clone();
                let index = children
                    .iter()
                    .position(|x| *x == node_id)
                    .expect("Should exist");
                if index > 0 {
                    return Some(children[index - 1]);
                }
            }
        }
        None
    }

    pub fn get_next_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(node_id) {
            let parent = node.parent;
            if parent.exists() {
                let children = self
                    .document
                    .nodes
                    .get(parent)
                    .as_ref()
                    .expect("Should exist")
                    .children
                    .clone();
                let index = children
                    .iter()
                    .position(|x| *x == node_id)
                    .expect("Should exist");
                if index < children.len() - 1 {
                    return Some(children[index + 1]);
                }
            }
        }
        None
    }

    pub fn select_next_in_tree(&mut self) {
        let next = self.get_next_in_tree(self.ui.selected_node);
        if let Some(next) = next {
            self.select_node(next);
        }
    }

    pub fn select_next_sibling(&mut self) {
        let next = self.get_next_sibling(self.ui.selected_node);
        if let Some(next) = next {
            self.select_node(next);
        }
    }

    fn get_next_in_tree(&self, node_id: NodeId) -> Option<NodeId> {
        if node_id.exists() {
            if let Some(node) = self.get(node_id) {
                if self.is_node_expanded(node_id) && !node.children.is_empty() {
                    return self.get_first_child(node_id);
                } else {
                    return self.get_next_tail(node_id);
                }
            }
        }
        None
    }

    fn get_next_tail(&self, node_id: NodeId) -> Option<NodeId> {
        if node_id.exists() {
            if let Some(node) = self.get(node_id) {
                if let Some(sibling) = self.get_next_sibling(node_id) {
                    return Some(sibling);
                } else if node.parent.exists() {
                    return self.get_next_tail(node.parent);
                }
            }
        }
        None
    }

    pub fn is_node_expanded(&self, node: NodeId) -> bool {
        node.is_root() || self.ui.expanded_nodes.contains(&node)
    }

    pub fn select_previous_sibling(&mut self) {
        let previous = self.get_previous_sibling(self.ui.selected_node);
        if let Some(previous) = previous {
            self.select_node(previous);
        }
    }

    pub fn select_previous_in_tree(&mut self) {
        let previous = self.get_previous_in_tree(self.ui.selected_node);
        if let Some(previous) = previous {
            self.select_node(previous);
        }
    }

    pub fn get_previous_in_tree(&self, node_id: NodeId) -> Option<NodeId> {
        if node_id.exists() {
            if let Some(sibling) = self.get_previous_sibling(node_id) {
                if self.is_node_expanded(sibling) && !self.get(sibling).unwrap().children.is_empty()
                {
                    return self.get_last_child(sibling);
                } else {
                    return Some(sibling);
                }
            } else {
                return self.get_parent(node_id);
            }
        }
        None
    }

    pub fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.get(node_id) {
            if node.parent.exists() {
                return Some(node.parent);
            }
        }
        None
    }

    pub fn select_parent(&mut self) {
        let selected_node = self.ui.selected_node;
        if selected_node.exists() {
            if let Some(node) = self.document.nodes.get(selected_node) {
                if node.parent.exists() {
                    self.select_node(node.parent);
                }
            }
        }
    }

    pub fn select_first_child(&mut self) {
        let selected_node = self.ui.selected_node;
        if selected_node.exists() {
            if let Some(first_child) = self.get_first_child(selected_node) {
                self.select_node(first_child);
            }
        }
    }

    pub fn get_first_child(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.get(node_id) {
            if !node.children.is_empty() {
                return Some(node.children[0]);
            }
        }
        None
    }

    pub fn get_last_child(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.get(node_id) {
            if !node.children.is_empty() {
                return Some(node.children[node.children.len() - 1]);
            }
        }
        None
    }

    pub fn toggle_selected_node_expanded(&mut self) {
        let selected_node = self.ui.selected_node;
        if selected_node.exists() {
            let is_expanded = self.ui.expanded_nodes.contains(&selected_node);
            self.set_node_expanded(selected_node, !is_expanded);
        }
    }

    pub fn set_node_expanded(&mut self, node: NodeId, expanded: bool) {
        if expanded {
            self.ui.expanded_nodes.insert(node);
        } else {
            self.ui.expanded_nodes.remove(&node);
        }
    }

    pub fn get(&self, node_id: NodeId) -> Option<&Node> {
        self.document.nodes.get(node_id)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.document.nodes.get_mut(node_id)
    }

    pub fn toggle_editing(&mut self) {
        self.ui.is_editing = !self.ui.is_editing;
    }
}

pub fn create_toolbar(app: &mut Application, ui: &mut Ui, extra: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                app.set_document(new_document());
            }
            if ui.button("Open…").clicked() {
                let result = open_document();
                if let Ok(Some((result, path))) = result {
                    app.set_document(result);
                    app.document_path = Some(path);
                } else {
                    show_error(result, "Failed to open document");
                }
            }
            if ui.button("Connect…").clicked() {
                app.ui.show_connect_dialog = true;
            }

            if ui.button("Save").clicked() {
                save_document(&mut app.document, app.document_path.clone());
            }
            if ui.button("Save as…").clicked() {
                save_document(&mut app.document, None);
            }

            ui.separator();

            ui.menu_button("Import…", |ui| {
                for importer in IMPORTERS {
                    if ui.button(importer.get_name()).clicked() {
                        let result = import_document_from_file(&importer);
                        if let Ok(Some(result)) = result {
                            app.set_document(result);
                        } else {
                            show_error(result, "Failed to import document");
                        }
                    }
                }
            });
        });

        ui.separator();

        if ui.button("↺").clicked() {
            app.document.undo();
        }

        let mut redo = Button::new("↻");
        if app.document.undo_changes.is_empty() {
            redo = redo.sense(Sense::empty());
        }

        if redo.ui(ui).clicked() {
            app.document.redo();
        }

        ui.separator();

        if ui.button("Commit").clicked() {
            app.commit("");
        }

        ui.spacing();

        if ui.button("🔅").clicked() {
            if ui.visuals().dark_mode {
                ui.ctx().set_visuals(egui::Visuals::light());
            } else {
                ui.ctx().set_visuals(egui::Visuals::dark());
            }
        }

        extra(ui);
    });

    if app.ui.show_connect_dialog {
        Modal::new(Id::new("Connect")).show(ui.ctx(), |ui| {
            ui.set_width(200.0);
            ui.heading("Connect to host");

            ui.text_edit_singleline(&mut app.ui.host_address)
                .on_hover_text("Host address");
            ui.add_space(8.0);
            if ui.button("Connect").clicked() {
                let url = app.ui.host_address.clone();
                app.connect_to_url(&url);
                app.ui.show_connect_dialog = false;
            }
        });
    }
}

pub fn show_error<T>(result: io::Result<T>, description: &str) {
    if let Err(error) = result {
        let text = format!("{}\n\n{}", description.to_string(), error.to_string());
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(text)
            .show();
    }
}

pub fn open_document() -> io::Result<Option<(Document, PathBuf)>> {
    let path = rfd::FileDialog::new()
        .add_filter("BINC files", &["binc"])
        .pick_file();

    if let Some(path) = path {
        let mut file = File::open(path.clone())?;
        let document = Document::read(&mut file)?;
        return Ok(Some((document, path)));
    }

    Ok(None)
}

pub fn import_document_from_file(importer: &Importer) -> io::Result<Option<Document>> {
    let extensions = importer.file_extensions();
    let path = rfd::FileDialog::new()
        .add_filter(importer.get_name(), &extensions)
        .pick_file();

    if let Some(path) = path {
        let mut file = File::open(path)?;
        let repo = importer.import(&mut file);
        let document = Document::new(repo?);
        return Ok(Some(document));
    }

    Ok(None)
}

pub fn save_document(
    document: &mut Document,
    known_path: Option<PathBuf>,
) -> io::Result<Option<PathBuf>> {
    if let Some(path) = known_path {
        let mut file = File::create(path.clone())?;
        document.write(&mut file)?;
        return Ok(Some(path));
    }

    let path = rfd::FileDialog::new()
        .add_filter("BINC files", &["binc"])
        .save_file();

    if let Some(path) = path {
        document.commit_changes();
        let mut file = File::create(path.clone())?;
        document.write(&mut file)?;
        return Ok(Some(path));
    }
    Ok(None)
}

pub fn new_document() -> Document {
    let mut document = Document::new(Repository::new());
    let id = document.next_id();
    let mut changes = Changes::new();
    changes
        .add_node(id, NodeId::ROOT_NODE, 0)
        .set_name(id, "First");
    let id = document.next_id();
    changes
        .add_node(id, NodeId::ROOT_NODE, 1)
        .set_name(id, "Second");
    let id = document.next_id();
    changes
        .add_node(id, NodeId::ROOT_NODE, 2)
        .set_name(id, "Third");
    document.add_and_apply_changes(changes);
    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use binc::node_id::NodeId;

    fn setup_app() -> Application {
        let mut app = Application::new();
        let mut changes = Changes::new();
        changes
            .add_node(NodeId::new(1), NodeId::ROOT_NODE, 0)
            .add_node(NodeId::new(2), NodeId::ROOT_NODE, 1)
            .add_node(NodeId::new(3), NodeId::new(1), 0);

        app.document.add_and_apply_changes(changes);
        app
    }

    #[test]
    fn test_select_next() {
        let mut app = setup_app();
        app.select_node(NodeId::ROOT_NODE);
        app.select_next_in_tree();
        assert_eq!(app.ui.selected_node, NodeId::new(1));
    }

    #[test]
    fn test_select_previous() {
        let mut app = setup_app();
        app.select_node(NodeId::new(2));
        app.select_previous_in_tree();
        assert_eq!(app.ui.selected_node, NodeId::new(1));
    }

    #[test]
    fn test_select_parent() {
        let mut app = setup_app();
        app.select_node(NodeId::new(1));
        app.select_parent();
        assert_eq!(app.ui.selected_node, NodeId::ROOT_NODE);
        app.select_node(NodeId::new(3));
        app.select_parent();
        assert_eq!(app.ui.selected_node.index(), 1);
    }

    #[test]
    fn test_select_first_child() {
        let mut app = setup_app();
        app.select_node(NodeId::ROOT_NODE);
        app.select_first_child();
        assert_eq!(app.ui.selected_node, NodeId::new(1));
    }

    #[test]
    fn test_select_impossible() {
        let mut app = setup_app();
        let ids = vec![
            NodeId::NO_NODE,
            NodeId::ROOT_NODE,
            NodeId::new(1),
            NodeId::new(2),
            NodeId::new(3),
        ];

        for id in ids {
            app.select_node(id);
            app.select_previous_in_tree();
            app.select_node(id);
            app.select_next_in_tree();
            app.select_node(id);
            app.select_parent();
            app.select_node(id);
            app.select_first_child();
        }
    }
}
