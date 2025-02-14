use crate::change::Change;
use crate::node_id::NodeId;

#[derive(Debug, Clone, Default)]
pub struct Changes {
    pub changes: Vec<Change>,
}

impl Changes {
    pub fn add_node(&mut self, id: NodeId, parent: NodeId, index_in_parent: u64) -> &mut Self {
        self.changes.push(Change::AddNode { id, parent, index_in_parent });
        self
    }

    pub fn remove_node(&mut self, id: NodeId) -> &mut Self {
        self.changes.push(Change::DeleteNode { id });
        self
    }

    pub fn move_node(&mut self, id: NodeId, new_parent: NodeId, index_in_new_parent: u64) -> &mut Self {
        self.changes.push(Change::MoveNode { id, new_parent, index_in_new_parent });
        self
    }

    pub fn set_string(&mut self, node: NodeId, attribute: String, value: String) -> &mut Self {
        self.changes.push(Change::SetString { node, attribute, value });
        self
    }

    pub fn set_bool(&mut self, node: NodeId, attribute: String, value: bool) -> &mut Self {
        self.changes.push(Change::SetBool { node, attribute, value });
        self
    }
}