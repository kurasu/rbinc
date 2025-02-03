use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::repository::Repository;

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
    pub children: Vec<Uuid>,
    pub attributes: HashMap<String, AttributeValue>,
}

impl Node {
    pub(crate) fn
    new() -> Node {
        Node {
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

    pub fn set_bool_attribute(&mut self, key: &String, value: bool) {
        self.attributes.insert(key.clone(), AttributeValue::Bool(value));
    }
}

pub struct Document {
    pub repository: Repository,
    pub nodes: HashMap<Uuid, Node>,
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
        Document { repository, nodes }
    }

    pub fn read(file: &mut dyn Read) -> io::Result<Document> {
        let repository = Repository::read(file)?;
        let nodes = compute_nodes(&repository);
        Ok(Document { repository, nodes })
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
}