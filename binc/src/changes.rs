use crate::attributes::AttributeValue;
use crate::node_id::NodeId;
use crate::operation::Operation;

#[derive(Debug, Clone, Default)]
pub struct Changes {
    pub operations: Vec<Operation>,
}

impl Changes {}

impl Changes {
    pub fn new() -> Changes {
        Changes::default()
    }
}

// TODO move this to journal and use that to easily create new operations

impl Changes {
    pub fn add_node(&mut self, id: NodeId, parent: NodeId, index_in_parent: usize) -> &mut Self {
        self.operations.push(Operation::AddNode {
            id,
            parent,
            index_in_parent,
        });
        self
    }

    pub fn remove_node(&mut self, id: NodeId) -> &mut Self {
        self.operations.push(Operation::RemoveNode { id });
        self
    }

    pub fn move_node(
        &mut self,
        id: NodeId,
        new_parent: NodeId,
        index_in_new_parent: usize,
    ) -> &mut Self {
        self.operations.push(Operation::MoveNode {
            id,
            new_parent,
            index_in_new_parent,
        });
        self
    }

    pub fn set_type_s(&mut self, node: NodeId, type_name: &str) -> &mut Self {
        let id = self.get_or_add_type_id(type_name);
        self.set_type(node, id)
    }

    pub fn set_type(&mut self, node: NodeId, type_id: usize) -> &mut Self {
        self.operations.push(Operation::SetType {
            node,
            type_id: type_id,
        });
        self
    }

    pub fn set_name(&mut self, node: NodeId, label: &str) -> &mut Self {
        self.operations.push(Operation::SetName {
            node,
            name: label.to_string(),
        });
        self
    }

    pub fn set_string_s(&mut self, node: NodeId, attribute: &str, value: &str) -> &mut Self {
        let id = self.get_or_add_attribute_id(attribute);
        self.set_string(node, id, value)
    }

    pub fn set_string(&mut self, node: NodeId, attribute: usize, value: &str) -> &mut Self {
        self.operations.push(Operation::SetAttribute {
            node,
            attribute,
            value: AttributeValue::String(value.to_string()),
        });
        self
    }

    pub fn set_bool(&mut self, node: NodeId, attribute: usize, value: bool) -> &mut Self {
        self.operations.push(Operation::SetAttribute {
            node,
            attribute,
            value: AttributeValue::Bool(value),
        });
        self
    }

    fn get_or_add_attribute_id(&mut self, attribute_name: &str) -> usize {
        let mut next_id = 0;
        for c in &self.operations {
            match c {
                Operation::DefineAttributeName { id, name } => {
                    if attribute_name == name {
                        return *id;
                    }
                    next_id = *id + 1;
                }
                _ => {}
            }
        }

        self.operations.push(Operation::DefineAttributeName {
            id: next_id,
            name: attribute_name.to_string(),
        });
        next_id
    }

    fn get_or_add_type_id(&mut self, type_name: &str) -> usize {
        let mut next_id = 0;
        for c in &self.operations {
            match c {
                Operation::DefineTypeName { id, name } => {
                    if type_name == name {
                        return *id;
                    }
                    next_id = *id + 1;
                }
                _ => {}
            }
        }

        self.operations.push(Operation::DefineTypeName {
            id: next_id,
            name: type_name.to_string(),
        });
        next_id
    }
}
