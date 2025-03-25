use crate::attributes::AttributeValue;
use crate::document::Document;
use crate::node_id::NodeId;
use crate::operation::Operation;

pub trait NodeBuilder {
    fn add_node(&mut self, type_name: &str, parent: NodeId) -> NodeId;
    fn insert_node(&mut self, type_name: &str, parent: NodeId, index: usize) -> NodeId;

    fn set_node_name(&mut self, node_id: NodeId, name: &str);
    fn set_node_type(&mut self, node_id: NodeId, type_name: &str);
    fn set_node_attribute_s(&mut self, node_id: NodeId, attribute: &str, name: &str);
    fn set_node_tag(&mut self, node_id: NodeId, tag: &str);
}

impl NodeBuilder for Document {
    fn add_node(&mut self, type_name: &str, parent: NodeId) -> NodeId {
        let id = self.node_id_generator.next_id();
        let index_in_parent = self
            .nodes
            .get(parent)
            .expect("Parent must exist")
            .children
            .len();
        let (node_type, exists) = self.nodes.type_names.get_or_create_index(type_name);
        if !exists {
            self.add_and_apply(Operation::DefineTypeName {
                id: node_type,
                name: type_name.to_string(),
            });
        }
        self.add_and_apply(Operation::AddNode {
            id,
            node_type,
            parent,
            index_in_parent,
        });
        id
    }

    fn insert_node(&mut self, type_name: &str, parent: NodeId, index: usize) -> NodeId {
        let id = self.node_id_generator.next_id();
        let (node_type, exists) = self.nodes.type_names.get_or_create_index(type_name);
        if !exists {
            self.add_and_apply(Operation::DefineTypeName {
                id: node_type,
                name: type_name.to_string(),
            });
        }
        self.add_and_apply(Operation::AddNode {
            id,
            node_type,
            parent,
            index_in_parent: index,
        });
        id
    }

    fn set_node_name(&mut self, node_id: NodeId, name: &str) {
        self.add_and_apply(Operation::SetName {
            node: node_id,
            name: name.to_string(),
        });
    }

    fn set_node_type(&mut self, node_id: NodeId, type_name: &str) {
        let t = self.nodes.type_names.get_index(type_name);

        let t = if t.is_none() {
            let new_id = self.nodes.type_names.len();
            self.add_and_apply(Operation::DefineTypeName {
                id: new_id,
                name: type_name.to_string(),
            });
            new_id
        } else {
            t.unwrap()
        };

        self.add_and_apply(Operation::SetType {
            node: node_id,
            type_id: t,
        });
    }

    fn set_node_attribute_s(&mut self, node_id: NodeId, attribute: &str, name: &str) {
        let attr = self.nodes.attribute_names.get_index(attribute);

        let attr = if attr.is_none() {
            let new_id = self.nodes.attribute_names.len();
            self.add_and_apply(Operation::DefineAttributeName {
                id: new_id,
                name: attribute.to_string(),
            });
            new_id
        } else {
            attr.unwrap()
        };

        self.add_and_apply(Operation::SetAttribute {
            node: node_id,
            attribute: attr,
            value: AttributeValue::String(name.to_string()),
        });
    }

    fn set_node_tag(&mut self, node_id: NodeId, tag: &str) {
        let t = self.nodes.tag_names.get_index(tag);

        let t = if t.is_none() {
            let new_id = self.nodes.tag_names.len();
            self.add_and_apply(Operation::DefineTagName {
                id: new_id,
                name: tag.to_string(),
            });
            new_id
        } else {
            t.unwrap()
        };

        self.add_and_apply(Operation::SetTag {
            node: node_id,
            tag: t,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_child() {
        let mut document = Document::default();
        let a = document.add_node("test", NodeId::ROOT_NODE);
        document.set_node_name(a, "hey");
        let b = document.add_node("test", NodeId::ROOT_NODE);
        document.set_node_name(b, "hey2");
        document.set_node_attribute_s(b, "speed", "high");
        assert_eq!(document.find_roots().len(), 2)
    }
}
