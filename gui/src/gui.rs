use std::collections::HashSet;
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
    pub selected_node: String,
    pub selected_node_name: String,
    pub expanded_nodes: HashSet<String>,
    pub is_editing: bool,
}

impl SimpleApplication {
    pub fn new() -> SimpleApplication {
        let mut app = SimpleApplication {
            document: Box::from(new_document()),
            selected_node: String::new(),
            selected_node_name: String::new(),
            expanded_nodes: HashSet::new(),
            is_editing: false,
        };
        //app.select_node(app.roots.get(0).);
        app
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Box::from(document);
        self.select_node("".to_string());
    }

    pub fn select_node(&mut self, path: String) {
        self.selected_node = path.clone();

        if !path.is_empty() {
            if let Some(node) = self.document.tree.get(&path) {
                self.selected_node_name = node.get_string_attribute("name").unwrap_or(String::new());
            }
        }
        else {
            self.selected_node_name = String::new();
        }
    }

    pub fn add_child(&mut self, path: &str) {
        let c = Change::AddNode { path: path.to_string() };
        self.document.add_and_apply_change(c);
    }

    pub fn remove_node(&mut self, path: &str) {
        let c = Change::AddNode { path: path.to_string() };
        self.document.add_and_apply_change(c);
        self.select_node("".to_string());
    }

    pub fn commit(&mut self) {
        self.document.commit_changes();
    }

    pub fn get_previous_sibling(&self, node_id: Uuid) -> Option<Uuid> {
        /*if let Some(node) = self.document.nodes.get(&node_id) {
            if let Some(parent) = node.parent {
                let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                let index = children.iter().position(|x| *x == node_id).expect("Should exist");
                if index > 0 {
                    return Some(children[index - 1]);
                }
            }
        }*/
        None
    }

    pub fn get_next_sibling(&self, node_id: Uuid) -> Option<Uuid> {
        /*if let Some(node) = self.document.nodes.get(&node_id) {
            if let Some(parent) = node.parent {
                let children = self.document.nodes.get(&parent).as_ref().expect("Should exist").children.clone();
                let index = children.iter().position(|x| *x == node_id).expect("Should exist");
                if index < children.len() - 1 {
                    return Some(children[index + 1]);
                }
            }
        }*/
        None
    }

    pub fn select_next(&mut self) {
        /*if let Some(selected_node) = self.selected_node {
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
        }*/
    }

    pub fn is_node_expanded(&self, path: String) -> bool {
        self.expanded_nodes.contains(&path)
    }

    pub fn select_previous(&mut self) {
        /*if let Some(selected_node) = self.selected_node {
            if let Some(sibling) = self.get_previous_sibling(selected_node) {
                if self.is_node_expanded(sibling) && !self.document.nodes.get(&sibling).unwrap().children.is_empty() {
                    self.select_node(self.get_last_child(sibling));
                } else {
                    self.select_node(Some(sibling));
                }
            } else {
                self.select_parent()
            }
        }*/
    }

    pub fn select_parent(&mut self) {
        if !self.selected_node.is_empty() {
            // strip last part of path
            let path = self.selected_node.split("/").collect::<Vec<&str>>()[0..self.selected_node.rfind('/').unwrap()].join("/");
            self.select_node(path);
        }
    }

    pub fn select_first_child(&mut self) {
        todo!()
    }

    pub fn get_first_child(&self, path: &str) -> Option<Uuid> {
        todo!()
    }

    pub fn get_last_child(&self, path: &str) -> Option<Uuid> {
        todo!()
    }

    pub fn toggle_selected_node_expanded(&mut self) {
        todo!()
    }

    pub fn set_node_expanded(&mut self, path: String, expanded: bool) {
        if expanded {
            self.expanded_nodes.insert(path);
        } else {
            self.expanded_nodes.remove(&path);
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
    let mut document = Document::default();
    document.add_and_apply_change(Change::AddNode { path: "first".to_string() });
    document
}