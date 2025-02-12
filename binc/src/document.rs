use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use crate::change::Change;
use crate::repository::Repository;
use crate::revision::Revision;
use crate::tree::Tree;

pub enum AttributeValue {
    String(String),
    Bool(bool),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{}", s),
            AttributeValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Default)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub attributes: HashMap<String, AttributeValue>
}

impl Node {

    pub fn new(name: &str) -> Node {
        Node {
            name: name.to_string(),
            children: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /*pub fn set_attribute<T>(&mut self, key: String, value: T) {
        self.attributes.insert(key, Box::new(value));
    }*/

    pub fn set_string_attribute(&mut self, key: &String, value: &String) {
        self.attributes.insert(key.clone(), AttributeValue::String(value.clone()));
    }

    pub fn get_string_attribute(&self, key: &str) -> Option<String> {
        match self.attributes.get(key) {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set_bool_attribute(&mut self, key: &String, value: bool) {
        self.attributes.insert(key.clone(), AttributeValue::Bool(value));
    }

    pub fn get_bool_attribute(&self, key: &str) -> Option<bool> {
        match self.attributes.get(key) {
            Some(AttributeValue::Bool(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Document {
    /// Repository containing all revisions
    pub repository: Repository,
    /// Root node of the document
    pub tree: Tree,
    /// Changes that have not been committed to the repository
    pub pending_changes: Box<Revision>,
    /// Changes that have been undone and can be redone
    pub undo_changes: Vec<Change>,
}

impl Document {

    fn build_tree(&mut self) -> Tree {
        let mut tree = Tree::default();

        for rev in &self.repository.revisions {
            for change in &rev.changes {
                change.apply(&mut tree).expect("Error applying change");
            }
        }
        for change in &self.pending_changes.changes {
            change.apply(&mut tree).expect("Error applying change");
        }

        tree
    }

    pub fn rebuild_tree(&mut self) {
        self.tree = self.build_tree();
    }

    pub fn new(repository: Repository) -> Document {
        let mut document = Self::default();
        document.repository = repository;
        document.rebuild_tree();
        document
    }

    pub fn read(file: &mut dyn Read) -> io::Result<Document> {
        let repository = Repository::read(file)?;
        Ok(Document::new(repository))
    }

    pub fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        self.repository.write(w)
    }

    pub fn add_and_apply_change(&mut self, change: Change) {
        self.undo_changes.clear();
        change.apply(&mut self.tree).expect("Error applying change");

        let last_change = self.pending_changes.changes.last();
        let combined_change = if last_change.is_some() { change.combine_change(last_change.unwrap()) } else { None };

        if let Some(combined_change) = combined_change {
            self.pending_changes.changes.pop();
            self.pending_changes.changes.push(combined_change);
        }
        else {
            self.pending_changes.changes.push(change);
        }
    }

    pub fn commit_changes(&mut self) {
        let pending = std::mem::replace(&mut self.pending_changes, Box::new(Revision::new()));
        self.repository.add_revision(*pending);
    }

    pub fn uncommit_changes(&mut self) {
        if self.pending_changes.changes.is_empty() && !self.repository.revisions.is_empty() {
            let last_revision = self.repository.revisions.pop().unwrap();
            self.pending_changes = Box::new(last_revision);
            self.rebuild_tree();
        }
    }

    pub fn undo(&mut self) {
        if self.pending_changes.changes.is_empty() {
            self.uncommit_changes();
        }

        if let Some(change) = self.pending_changes.changes.pop() {
            self.undo_changes.push(change);
        }

        self.rebuild_tree();
    }

    pub fn redo(&mut self) {
        if let Some(change) = self.undo_changes.pop() {
            self.pending_changes.changes.push(change);
            self.rebuild_tree();
        }
    }
}