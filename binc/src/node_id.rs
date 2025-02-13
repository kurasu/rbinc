use std::fmt::Display;
use std::{io, vec};
use crate::change::Change;
use crate::document::Node;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct NodeId {
    pub (crate) id: usize,
}

impl NodeId {
    const ROOT_NODE: usize = 0;

    pub fn index(&self) -> usize {
        self.id
    }

    pub fn new(id: usize) -> NodeId {
        NodeId { id }
    }

    pub fn is_root(&self) -> bool {
        self.id == Self::ROOT_NODE
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId { id: get_next_id() }
    }
}

fn get_next_id() -> usize {
    static mut NEXT_ID: usize = 1;
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