use std::collections::HashSet;
use std::fs::File;
use std::io;
use eframe::egui::{Button, Sense, Ui, Widget};
use rfd::MessageLevel::Error;
use binc::document::Document;
use binc::repository::Repository;
use binc::change::Change;
use binc::id::NodeId;

pub struct SimpleApplication {
    pub document: Box<Document>,
    pub roots: Vec<NodeId>,
    pub selected_node: Option<NodeId>,
    pub selected_node_name: String,
    pub expanded_nodes: HashSet<NodeId>,
    pub is_editing: bool,
}

impl SimpleApplication {
    pub fn new() -> SimpleApplication {
        let mut app = SimpleApplication {
            document: Box::from(new_document()),
            roots: Vec::new(),
            selected_node: None,
            selected_node_name: String::new(),
            expanded_nodes: HashSet::new(),
            is_editing: false,
        };
        app.roots = app.document.find_roots();
        //app.select_node(app.roots.get(0).);
        app
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Box::from(document);
        self.roots = self.document.find_roots();
        self.select_node(None);
    }

    pub fn select_node(&mut self, node_id: Option<NodeId>) {
        self.selected_node = node_id;
        if let Some(node_id) = node_id {
            let name = self.document.nodes.get(&node_id).as_ref().expect("Should exist").get_string_attribute("name");
            self.selected_node_name = name.unwrap_or(String::new());
        }
        else {
            self.selected_node_name = String::new();
        }
    }

    pub fn add_child(&mut self, parent_id: &NodeId, insertion_index: u64) {
        let child_id = NodeId::default();

        let c1 = Change::AddNode { id: child_id };
        let c2 = Change::AddChild { parent: parent_id.clone(), child: child_id, insertion_index: insertion_index };
        self.document.add_and_apply_change(c1);
        self.document.add_and_apply_change(c2);
    }

    pub fn remove_node(&mut self, node_id: &NodeId) {
        let mut changes : Vec<Change> = vec![];
        if let Some(node) = self.document.nodes.get(node_id) {
            if let Some(parent) = node.parent {
                changes.push(Change::RemoveChild { parent: parent.clone(), child: node_id.clone() });
            }
        }
        changes.push(Change::RemoveNode { id: node_id.clone() });
        for c in changes {
            self.document.add_and_apply_change(c);
        }
        self.select_node(None);
        self.roots = self.document.find_roots();
    }

    pub fn commit(&mut self) {
        self.document.commit_changes();
    }

    pub fn get_previous_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(&node_id) {
            if let Some(parent) = node.parent {
                let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                let index = children.iter().position(|x| *x == node_id).expect("Should exist");
                if index > 0 {
                    return Some(children[index - 1]);
                }
            }
        }
        None
    }

    pub fn get_next_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(&node_id) {
            if let Some(parent) = node.parent {
                let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                let index = children.iter().position(|x| *x == node_id).expect("Should exist");
                if index < children.len() - 1 {
                    return Some(children[index + 1]);
                }
            }
        }
        None
    }

    pub fn select_next(&mut self) {
        if let Some(selected_node) = self.selected_node {
            if self.expanded_nodes.contains(&selected_node) {
                self.select_first_child();
            }
            else if let Some(node) = self.document.nodes.get(&selected_node) {
                if let Some(sibling) = self.get_next_sibling(selected_node) {
                    self.select_node(Some(sibling));
                }
                else if let Some(parent) = node.parent {
                    self.select_node(self.get_next_sibling(parent));
                }
            }
        }
    }

    pub fn is_node_expanded(&self, node: NodeId) -> bool {
        self.expanded_nodes.contains(&node)
    }

    pub fn select_previous(&mut self) {
        if let Some(selected_node) = self.selected_node {
            if let Some(sibling) = self.get_previous_sibling(selected_node) {
                if self.is_node_expanded(sibling) && !self.document.nodes.get(&sibling).unwrap().children.is_empty() {
                    self.select_node(self.get_last_child(sibling));
                } else {
                    self.select_node(Some(sibling));
                }
            } else {
                self.select_parent()
            }
        }
    }

    pub fn select_parent(&mut self) {
        if let Some(selected_node) = self.selected_node {
            if let Some(node) = self.document.nodes.get(&selected_node) {
                if let Some(parent) = node.parent {
                    self.select_node(Some(parent));
                }
            }
        }
    }

    pub fn select_first_child(&mut self) {
        if let Some(selected_node) = self.selected_node {
            self.set_node_expanded(selected_node, true);
            if let Some(node) = self.document.nodes.get(&selected_node) {
                if !node.children.is_empty() {
                    self.select_node(Some(node.children[0]));
                }
            }
        }
    }

    pub fn get_first_child(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(&node_id) {
            if !node.children.is_empty() {
                return Some(node.children[0]);
            }
        }
        None
    }

    pub fn get_last_child(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.document.nodes.get(&node_id) {
            if !node.children.is_empty() {
                return Some(node.children[node.children.len() - 1]);
            }
        }
        None
    }

    pub fn toggle_selected_node_expanded(&mut self) {
        if let Some(selected_node) = self.selected_node {
            let is_expanded = self.expanded_nodes.contains(&selected_node);
            self.set_node_expanded(selected_node, !is_expanded);
        }
    }

    pub fn set_node_expanded(&mut self, node: NodeId, expanded: bool) {
        if expanded {
            self.expanded_nodes.insert(node);
        } else {
            self.expanded_nodes.remove(&node);
        }
    }

    pub fn toggle_editing(&mut self) {
        self.is_editing = !self.is_editing;
    }
}

pub fn create_toolbar(app: &mut SimpleApplication, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui.button("New").clicked() {
            app.set_document(new_document());
        }
        if ui.button("Open").clicked() {
            let result = open_document();
            if let Ok(Some(result)) = result {
                app.set_document(result);
            } else { show_error(result, "Failed to open document"); }
        }
        if ui.button("Save").clicked() {
            save_document(&mut app.document);
        }

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
    });
}

pub fn show_error<T>(result: io::Result<T>, description: &str) {
    if let Err(error) = result {
        let text = format!("{}\n\n{}", description.to_string(), error.to_string());
        rfd::MessageDialog::new().set_level(Error).set_title("Error").set_description(text).show();
    }
}

pub fn open_document() -> io::Result<Option<Document>> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).pick_file();

    if let Some(path) = path {
        let mut file = File::open(path)?;
        let document = Document::read(&mut file)?;
        return Ok(Some(document));
    }

    Ok(None)
}

pub fn save_document(document: &mut Document) -> io::Result<bool> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).save_file();

    if let Some(path) = path {
        document.commit_changes();
        let mut file = File::create(path)?;
        document.write(&mut file)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn new_document() -> Document {
    let mut document = Document::new(Repository::new());
    let root = NodeId::default();
    document.add_and_apply_change(Change::AddNode { id: root.clone() });
    document.add_and_apply_change(Change::SetString { node: root, attribute: "name".to_string(), value: "Root".to_string() });
    document
}