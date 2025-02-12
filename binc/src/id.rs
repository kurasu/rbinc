use std::fmt::Display;
use std::vec;
use crate::document::Node;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct NodeId {
    pub (crate) id: u32,
}

impl NodeId {
    fn index(&self) -> usize {
        self.id as usize
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId { id: get_next_id() }
    }
}

fn get_next_id() -> u32 {
    static mut NEXT_ID: u32 = 0;
    unsafe {
        NEXT_ID += 1;
        NEXT_ID
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub struct NodeStore {
    nodes: Vec<Option<Node>>
}

impl NodeStore {
    pub fn new() -> NodeStore {
        NodeStore {
            nodes: vec![]
        }
    }

    pub fn find_roots(&self) -> Vec<NodeId> {
        let mut roots: Vec<NodeId> = vec![];
        for node in &self.nodes {
            if let Some(node) = node {
                if node.parent.is_none() {
                    roots.push(node.id);
                }
            }
        }
        roots
    }

    pub fn insert(&mut self, id: &NodeId, node: Node) -> Option<Node> {
        let index = id.index();
        if index >= self.nodes.len() {
            self.nodes.resize_with(32 + index - self.nodes.len(), || None);
        }
        let old = self.nodes.remove(index);
        self.nodes[index] = Some(node);
        old
    }

    pub fn remove(&mut self, id: &NodeId) -> Option<Node> {
        self.nodes.remove(id.index())
    }

    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id.index()).unwrap().as_ref()
    }

    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index()).unwrap().as_mut()
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}