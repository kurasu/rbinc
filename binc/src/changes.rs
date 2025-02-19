use crate::attributes::AttributeValue;
use crate::change::Change;
use crate::node_id::NodeId;

#[derive(Debug, Clone, Default)]
pub struct Changes {
    pub changes: Vec<Change>,
}

impl Changes {
}

impl Changes {
    pub fn new() -> Changes {
        Changes::default()
    }
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

    pub fn set_type(&mut self, node: NodeId, type_name: &str) -> &mut Self {
        self.changes.push(Change::SetType { node, type_name: type_name.to_string() });
        self
    }

    pub fn set_name(&mut self, node: NodeId, label: &str) -> &mut Self {
        self.changes.push(Change::SetName { node, name: label.to_string() });
        self
    }

    pub fn set_string(&mut self, node: NodeId, attribute: &str, value: &str) -> &mut Self {
        self.changes.push(Change::SetAttribute { node, attribute: attribute.to_string(), value: AttributeValue::String(value.to_string()) });
        self
    }

    pub fn set_bool(&mut self, node: NodeId, attribute: &str, value: bool) -> &mut Self {
        self.changes.push(Change::SetAttribute { node, attribute: attribute.to_string(), value: AttributeValue::Bool(value) });
        self
    }
}