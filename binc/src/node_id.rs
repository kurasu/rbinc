use std::fmt::Display;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct NodeId {
    pub(crate) id: usize,
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId {
            id: Self::NO_NODE_ID,
        }
    }
}

impl NodeId {
    pub const ROOT_NODE_ID: usize = 0;
    pub const NO_NODE_ID: usize = 0xFFFFFFFF;
    pub const ROOT_NODE: NodeId = NodeId {
        id: Self::ROOT_NODE_ID,
    };
    pub const NO_NODE: NodeId = NodeId {
        id: Self::NO_NODE_ID,
    };

    pub fn index(&self) -> usize {
        self.id
    }

    pub fn new(id: usize) -> NodeId {
        assert_ne!(id, Self::NO_NODE_ID);
        assert_ne!(id, Self::ROOT_NODE_ID);
        NodeId { id }
    }

    pub fn is_root(&self) -> bool {
        self.id == Self::ROOT_NODE_ID
    }

    pub fn exists(&self) -> bool {
        self.id != Self::NO_NODE_ID
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct NodeIdGenerator {
    next_id: usize,
}

impl Default for NodeIdGenerator {
    fn default() -> Self {
        NodeIdGenerator::new()
    }
}

impl NodeIdGenerator {
    pub fn new() -> NodeIdGenerator {
        NodeIdGenerator { next_id: 1 }
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId::new(id)
    }
}
