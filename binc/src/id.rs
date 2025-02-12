use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::Display;
use uuid::Uuid;
use crate::document::Node;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct NodeId {
    pub (crate) id: Uuid,
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId { id: Uuid::new_v4() }
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub struct NodeStore {
    map: HashMap<NodeId, Node>
}

impl NodeStore {
    pub fn new() -> NodeStore {
        NodeStore {
            map: HashMap::new()
        }
    }

    pub fn find_roots(&self) -> Vec<NodeId> {
        let mut roots: Vec<NodeId> = vec![];
        for (_id, node) in &self.map {
            if node.parent.is_none() {
                roots.push(node.id.clone());
            }
        }
        roots
    }

    pub fn insert(&mut self, id: &NodeId, node: Node) -> Option<Node> {
        self.map.insert(id.clone(), node)
    }

    pub fn remove(&mut self, id: &NodeId) -> Option<Node> {
        self.map.remove(id)
    }

    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.map.get_mut(id)
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}