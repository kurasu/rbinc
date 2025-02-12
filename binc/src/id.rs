use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::Display;
use uuid::Uuid;
use crate::document::Node;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct Id {
    pub (crate) id: Uuid,
}

impl Default for Id {
    fn default() -> Self {
        Id { id: Uuid::new_v4() }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub struct IdStore {
    map: HashMap<Id, Node>
}

impl IdStore {
    pub fn new() -> IdStore {
        IdStore {
            map: HashMap::new()
        }
    }

    pub fn find_roots(&self) -> Vec<Id> {
        let mut roots: Vec<Id> = vec![];
        for (_id, node) in &self.map {
            if node.parent.is_none() {
                roots.push(node.id.clone());
            }
        }
        roots
    }

    pub fn insert(&mut self, id: &Id, node: Node) -> Option<Node> {
        self.map.insert(id.clone(), node)
    }

    pub fn remove(&mut self, id: &Id) -> Option<Node> {
        self.map.remove(id)
    }

    pub fn get(&self, id: &Id) -> Option<&Node> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: &Id) -> Option<&mut Node> {
        self.map.get_mut(id)
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}