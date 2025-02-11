use std::fs::File;
use std::io;
use eframe::egui::{Button, Sense, Ui, Widget};
use rfd::MessageLevel::Error;
use binc::document::Document;
use binc::repository::Repository;
use uuid::Uuid;
use binc::change::Change;

pub struct SimpleApplication {
    pub document: Box<Document>,
    pub roots: Vec<Uuid>,
    pub selected_node: Option<Uuid>,
    pub selected_node_name: String,
}

impl SimpleApplication {
    pub fn new() -> SimpleApplication {
        SimpleApplication {
            document: Box::from(new_document()),
            roots: Vec::new(),
            selected_node: None,
            selected_node_name: String::new(),
        }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Box::from(document);
        self.roots = self.document.find_roots();
        self.select_node(None);
    }

    pub fn select_node(&mut self, node_id: Option<Uuid>) {
        self.selected_node = node_id;
        if let Some(node_id) = node_id {
            let name = self.document.nodes.get(&node_id).as_ref().expect("Should exist").get_string_attribute("name");
            self.selected_node_name = name.unwrap_or(String::new());
        }
        else {
            self.selected_node_name = String::new();
        }
    }

    pub fn add_child(&mut self, parent_id: &Uuid, insertion_index: u64) {
        let child_id = Uuid::new_v4();

        let c1 = Change::AddNode { uuid: child_id };
        let c2 = Change::AddChild { parent: parent_id.clone(), child: child_id, insertion_index: insertion_index };
        self.document.add_and_apply_change(c1);
        self.document.add_and_apply_change(c2);
    }

    pub fn remove_node(&mut self, node_id: &Uuid) {
        let mut changes : Vec<Change> = vec![];
        for v in self.document.nodes.values() {
            if v.children.contains(node_id) {
                changes.push(Change::RemoveChild { parent: v.uuid.clone(), child: node_id.clone() });
            }
        }
        changes.push(Change::RemoveNode { uuid: node_id.clone() });
        for c in changes {
            self.document.add_and_apply_change(c);
        }
        self.select_node(None);
        self.roots = self.document.find_roots();
    }

    pub fn commit(&mut self) {
        self.document.commit_changes();
    }

    pub fn select_next_sibling(&mut self) {
        if let Some(selected_node) = self.selected_node {
            if let Some(node) = self.document.nodes.get(&selected_node) {
                if let Some(parent) = node.parent {
                    let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                    let index = children.iter().position(|x| *x == selected_node).expect("Should exist");
                    if index < children.len() - 1 {
                        self.select_node(Some(children[index + 1]));
                    }
                }
            }
        }
    }

    pub fn select_previous_sibling(&mut self) {
        if let Some(selected_node) = self.selected_node {
            if let Some(node) = self.document.nodes.get(&selected_node) {
                if let Some(parent) = node.parent {
                    let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                    let index = children.iter().position(|x| *x == selected_node).expect("Should exist");
                    if index > 0 {
                        self.select_node(Some(children[index - 1]));
                    }
                }
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
            if let Some(node) = self.document.nodes.get(&selected_node) {
                if !node.children.is_empty() {
                    self.select_node(Some(node.children[0]));
                }
            }
        }
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
            save_document(&app.document);
        }

        ui.separator();

        if ui.button("Undo").clicked() {
            app.document.undo();
        }

        let mut redo = Button::new("Redo");
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

pub fn save_document(document: &Document) -> io::Result<bool> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).save_file();

    if let Some(path) = path {
        let mut file = File::create(path)?;
        document.write(&mut file)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn new_document() -> Document {
    Document::new(Repository::new())
}