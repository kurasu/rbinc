use std::collections::HashMap;
use crate::document::{AttributeValue};
use crate::node_id::NodeId;

pub type NodeStore = FlatNodeStore;

struct FlatNodeStore {
    nodes: Vec<Option<Node>>
}

impl FlatNodeStore {
    pub fn new() -> NodeStore {
        NodeStore {
            nodes: vec![]
        }
    }

    pub fn find_roots(&self) -> Vec<NodeId> {
        self.nodes.get(0).expect("Root node should exist").get
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



pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub attributes: HashMap<String, AttributeValue>
}

impl Node {

    pub fn new_with_id(id: &NodeId) -> Node {
        Node {
            id: id.clone(),
            parent: None,
            children: vec![],
            attributes: HashMap::new(),
        }
    }

    /*pub fn set_attribute<T>(&mut self, key: String, value: T) {
        self.attributes.insert(key, Box::new(value));
    }*/

    pub fn set_string_attribute(&mut self, key: &String, value: &String) {
        self.attributes.insert(key.clone(), AttributeValue::String(value.clone()));
    }

    pub fn get_string_attribute(&self, key: &str) -> Option<String> {
        match self.attributes.get(key) {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set_bool_attribute(&mut self, key: &String, value: bool) {
        self.attributes.insert(key.clone(), AttributeValue::Bool(value));
    }

    pub fn get_bool_attribute(&self, key: &str) -> Option<bool> {
        match self.attributes.get(key) {
            Some(AttributeValue::Bool(s)) => Some(s.clone()),
            _ => None,
        }
    }
}