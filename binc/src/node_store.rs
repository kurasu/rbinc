use std::ops::Deref;
use crate::document::{AttributeValue};
use crate::node_id::NodeId;

pub type NodeStore = FlatNodeStore;

pub struct FlatNodeStore {
    nodes: Vec<Node>
}

impl FlatNodeStore {
    pub fn new() -> NodeStore {
        let mut nodes = vec![Node::default()];
        nodes[0].id = NodeId::ROOT_NODE;
        nodes[0].parent = NodeId::NO_NODE;

        NodeStore { nodes }
    }

    pub fn find_roots(&self) -> &Vec<NodeId> {
        let x = self.nodes.get(0).expect("Root node should exist");
        x.children.as_ref()
    }

    pub fn exists(&self, id: NodeId) -> bool {
        self.nodes.get(id.index()).is_some()
    }

    pub(crate) fn add(&mut self, id: NodeId, parent: NodeId, index_in_parent: usize) {
        let i = id.index();
        let p = parent.index();

        if i >= self.nodes.len() {
            self.nodes.resize_with(i + 1, || Node::default());
        }

        self.nodes[i] = Node::new_with_id(id.clone());
        self.nodes[p].children.insert(index_in_parent, id.clone());
    }

    pub(crate) fn delete_recursive(&mut self, id: NodeId) {
        let i = id.index();
        for c in self.nodes[i].children.clone() {
            self.delete_recursive(c);
        }
        let p = self.nodes[i].parent.index();
        self.nodes[p].children.retain(|x| *x != id);
        self.nodes[i] = Node::default();
    }

    pub (crate) fn move_node(&mut self, id: NodeId, new_parent: NodeId, index_in_new_parent: usize) {
        let i = id.index();
        let p1 = self.nodes[i].parent.index();
        let p2 = new_parent.index();

        self.nodes[p1].children.retain(|x| *x != id);
        self.nodes[p2].children.insert(index_in_new_parent, id.clone());
        self.nodes[i].parent = new_parent.clone();
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index())
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}


pub struct Node {
    pub id: NodeId,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
    pub attributes: AttributeStore
}

#[derive(Debug, Clone, Default)]
pub struct AttributeStore {
    attributes: Vec<AttributeEntry>
}

#[derive(Debug, Clone)]
pub struct AttributeEntry {
    pub key: String,
    pub value: AttributeValue
}

impl AttributeStore {
    pub fn set(&mut self, key: &str, value: AttributeValue) {
        for a in &mut self.attributes {
            if a.key == key {
                a.value = value;
                return;
            }
        }

        self.attributes.push(AttributeEntry { key: key.to_string(), value });
    }

    pub fn get(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.iter().find(|x| x.key == key).map(|x| &x.value)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue> {
        self.attributes.iter_mut().find(|x| x.key == key).map(|x| &mut x.value)
    }

    pub fn iter(&self) -> std::slice::Iter<AttributeEntry> {
        self.attributes.iter()
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            id: NodeId::NO_NODE,
            parent: NodeId::NO_NODE,
            children: vec![],
            attributes: AttributeStore::default(),
        }
    }
}

impl Node {

    pub fn new_with_id(id: NodeId) -> Node {
        Node {
            id,
            parent: NodeId::ROOT_NODE,
            children: vec![],
            attributes: AttributeStore::default(),
        }
    }

    /*pub fn set_attribute<T>(&mut self, key: String, value: T) {
        self.attributes.insert(key, Box::new(value));
    }*/

    pub fn set_attribute(&mut self, key: &str, value: AttributeValue) {
        self.attributes.set(key, value);
    }

    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }

    pub fn set_string_attribute(&mut self, key: &String, value: &String) {
        self.set_attribute(key, AttributeValue::String(value.clone()));
    }

    pub fn set_bool_attribute(&mut self, key: &String, value: bool) {
        self.set_attribute(key, AttributeValue::Bool(value));
    }

    pub fn get_string_attribute(&self, key: &str) -> Option<String> {
        match self.attributes.get(key) {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_bool_attribute(&self, key: &str) -> Option<bool> {
        match self.attributes.get(key) {
            Some(AttributeValue::Bool(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node_store() {
        let store = FlatNodeStore::new();
        assert_eq!(store.nodes[0].parent, NodeId::NO_NODE);
        assert_eq!(store.nodes[0].id, NodeId::ROOT_NODE);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_insert_and_get_node() {
        let mut store = FlatNodeStore::new();
        let node_id = NodeId::new(1);
        store.add(node_id, NodeId::ROOT_NODE, 0);
        assert!(store.get(node_id).is_some());
        assert!(store.get(node_id).expect("Node not found").parent == NodeId::ROOT_NODE);
        assert!(store.get(node_id).expect("Node not found").id == node_id);
    }

    #[test]
    fn test_set_and_get_attribute() {
        let mut node = Node::new_with_id(NodeId::new(1));
        node.set_string_attribute(&"key".to_string(), &"value".to_string());
        assert_eq!(node.get_string_attribute("key"), Some("value".to_string()));
    }

    #[test]
    fn test_find_roots() {
        let mut store = FlatNodeStore::new();
        let root_node = Node::new_with_id(NodeId::new(0));
        store.nodes.push(root_node);
        let roots = store.find_roots();
        assert_eq!(roots.len(), 0);
        let node_id = NodeId::new(1);
        store.add(node_id, NodeId::ROOT_NODE, 0);
        let roots = store.find_roots();
        assert_eq!(roots.len(), 1);
        assert!(roots[0] == node_id);
    }
}