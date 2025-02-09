use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::change::Change;
use crate::repository::Repository;
use crate::revision::Revision;

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

pub struct Node {
    pub uuid: Uuid,
    pub children: Vec<Uuid>,
    pub attributes: HashMap<String, AttributeValue>
}

impl Node {
    pub(crate) fn
    new() -> Node {
        Node {
            uuid: Uuid::new_v4(),
            children: vec![],
            attributes: HashMap::new(),
        }
    }

    pub fn new_with_uuid(uuid: Uuid) -> Node {
        Node {
            uuid: uuid,
            children: vec![],
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
}

pub struct Document {
    pub repository: Repository,
    pub nodes: HashMap<Uuid, Node>,
    pub pending_changes: Box<Revision>,
}

fn compute_nodes(repository: &Repository) -> HashMap<Uuid, Node> {
    let mut nodes: HashMap<Uuid, Node> = HashMap::new();
    for rev in &repository.revisions {
        for change in &rev.changes {
            change.apply(&mut nodes);
        }
    }
    nodes
}

impl Document {
    pub fn new(repository: Repository) -> Document {
        let nodes = compute_nodes(&repository);
        Document { repository, nodes, pending_changes: Box::new(Revision::new()) }
    }

    pub fn read(file: &mut dyn Read) -> io::Result<Document> {
        let repository = Repository::read(file)?;
        let nodes = compute_nodes(&repository);
        Ok(Document { repository, nodes, pending_changes: Box::new(Revision::new()) })
    }

    pub fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        self.repository.write(w)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn find_roots(&self) -> Vec<Uuid> {
        let mut roots: Vec<&Uuid> = self.nodes.keys().collect();
        for (_uuid, node) in &self.nodes {
            for child in &node.children {
                roots.retain(|&x| x != child);
            }
        }
        roots.drain(..).map(|x| *x).collect()
    }

    pub fn add_and_apply_change(&mut self, change: Change) {
        change.apply(&mut self.nodes);

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
}

fn can_combine(p0: &Change, p1: &Change) -> bool {
    todo!()
}