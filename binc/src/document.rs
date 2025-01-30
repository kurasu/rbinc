use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;
use crate::repository::Repository;
use crate::revision::Change;

pub struct Node {
    pub children: Vec<Node>,
    pub attributes: HashMap<String, Box<dyn Any>>,
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
        self.attributes.insert(key.clone(), Box::new(value.clone()));
    }
}

pub struct Document {
    pub repository: Repository,
    nodes: HashMap<Uuid, Node>,
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

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}