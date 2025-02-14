use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use crate::node_id::NodeId;
use crate::readwrite::{ReadExt, WriteExt};
use crate::node_store::{Node, NodeStore};

pub(crate) struct ChangeType;

pub trait OptionExt {
    fn expect_none(&self, msg: &str);
}

impl<T> OptionExt for Option<T> {
    fn expect_none(&self, msg: &str) {
        if self.is_some() {
            panic!("{}", msg);
        }
    }
}

impl ChangeType {
    pub const ADD_NODE: u64 = 0x01;
    pub const ADD_NODE_FROM_SOURCE: u64 = 0x02;
    pub const MOVE_NODE: u64 = 0x04;
    pub const REMOVE_NODE: u64 = 0x08;

    pub const ADD_CHILD: u64 = 0x11;
    pub const REMOVE_CHILD: u64 = 0x12;

    pub const ADD_SOURCE: u64 = 0x21;
    pub const UPDATE_SOURCE: u64 = 0x22;
    pub const REMOVE_SOURCE: u64 = 0x23;

    pub const ADD_COMMENT: u64 = 0x31;

    pub const SET_BOOL: u64 = 0x40;
    pub const SET_STRING: u64 = 0x41;
    pub const SET_UUID: u64 = 0x42;
    pub const SET_UINT8: u64 = 0x43;
    pub const SET_UINT16: u64 = 0x44;
    pub const SET_UINT32: u64 = 0x45;
    pub const SET_UINT64: u64 = 0x46;
    pub const SET_INT8: u64 = 0x47;
    pub const SET_INT16: u64 = 0x48;
    pub const SET_INT32: u64 = 0x49;
    pub const SET_INT64: u64 = 0x4A;
    pub const SET_FLOAT16: u64 = 0x4B;
    pub const SET_FLOAT32: u64 = 0x4C;
    pub const SET_FLOAT64: u64 = 0x4D;
    pub const SET_FLOAT80: u64 = 0x4E;

    pub const SET_BOOL_ARRAY: u64 = 0x60;
    pub const SET_STRING_ARRAY: u64 = 0x61;
    pub const SET_UUID_ARRAY: u64 = 0x62;
    pub const SET_UINT8_ARRAY: u64 = 0x63;
    pub const SET_UINT16_ARRAY: u64 = 0x64;
    pub const SET_UINT32_ARRAY: u64 = 0x65;
    pub const SET_UINT64_ARRAY: u64 = 0x66;
    pub const SET_INT8_ARRAY: u64 = 0x67;
    pub const SET_INT16_ARRAY: u64 = 0x68;
    pub const SET_INT32_ARRAY: u64 = 0x69;
    pub const SET_INT64_ARRAY: u64 = 0x6A;
    pub const SET_FLOAT16_ARRAY: u64 = 0x6B;
    pub const SET_FLOAT32_ARRAY: u64 = 0x6C;
    pub const SET_FLOAT64_ARRAY: u64 = 0x6D;
    pub const SET_FLOAT80_ARRAY: u64 = 0x6E;

    pub const UNKNOWN: u64 = 0x7FFFFE; // Only used internally
    pub const ERROR: u64 = 0x7FFFFF; // Only used internally
}

#[derive(Debug, Clone)]
pub enum Change {
    AddNode {id: NodeId, parent: NodeId, index_in_parent: u64},
    MoveNode {id: NodeId, new_parent: NodeId, index_in_new_parent: u64},
    DeleteNode {id: NodeId },
    SetString {node: NodeId, attribute: String, value: String},
    SetBool {node: NodeId, attribute: String, value: bool},
    UnknownChange {change_type: u64, data: Vec<u8>},
}

impl Change {
    pub(crate) fn apply(&self, nodes: &mut NodeStore)
    {
        match self {
            Change::AddNode {id, parent, index_in_parent} => {
                nodes.add(*id, *parent, *index_in_parent as usize);
            }
            Change::DeleteNode { id } => {
                nodes.delete_recursive(*id);
            }
            Change::MoveNode {id, new_parent, index_in_new_parent} => {
                nodes.move_node(*id, *new_parent, *index_in_new_parent as usize);
            }
            Change::SetString {node, attribute, value} => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_string_attribute(attribute, value);
            }
            Change::SetBool {node, attribute, value} => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_bool_attribute(attribute, *value);
            }
            Change::UnknownChange {change_type: _, data: _} => {
                // Do nothing
            }
        }
    }

    pub(crate) fn read(mut r: &mut dyn Read) -> io::Result<Change> {
        let change_type = r.read_length()?;
        let change_size = r.read_length()?;
        match change_type {
            ChangeType::ADD_NODE => {
                let id = r.read_id()?;
                let parent = r.read_id()?;
                let index_in_parent = r.read_length()?;
                Ok(Change::AddNode {id, parent, index_in_parent})
            }
            ChangeType::REMOVE_NODE => {
                let node = r.read_id()?;
                Ok(Change::DeleteNode {id: node})
            }
            ChangeType::MOVE_NODE => {
                let id = r.read_id()?;
                let new_parent = r.read_id()?;
                let index_in_new_parent = r.read_length()?;
                Ok(Change::MoveNode {id, new_parent, index_in_new_parent})
            }
            ChangeType::SET_STRING => {
                let node = r.read_id()?;
                let attribute = r.read_string()?;
                let value = r.read_string()?;
                Ok(Change::SetString {node, attribute, value})
            }
            ChangeType::SET_BOOL => {
                let node = r.read_id()?;
                let attribute = r.read_string()?;
                let value = r.read_u8()? != 0;
                Ok(Change::SetBool {node, attribute, value})
            }
            _ => {
                let mut data = vec![0; change_size as usize];
                r.read_exact(&mut data)?;
                Ok(Change::UnknownChange {change_type, data})
            }
        }
    }

    pub(crate) fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        match self {
            Change::AddNode {id, parent, index_in_parent} => {
                w.write_id(id)?;
                w.write_id(parent)?;
                w.write_length(*index_in_parent)
            }
            Change::MoveNode {id, new_parent, index_in_new_parent} => {
                w.write_id(id)?;
                w.write_id(new_parent)?;
                w.write_length(*index_in_new_parent)
            }
            Change::DeleteNode {id} => {
                w.write_id(id)
            }
            Change::SetString {node, attribute, value} => {
                w.write_id(node)?;
                w.write_string(attribute)?;
                w.write_string(value)
            }
            Change::SetBool {node, attribute, value} => {
                w.write_id(node)?;
                w.write_string(attribute)?;
                w.write_u8(*value as u8)
            }
            Change::UnknownChange {change_type: _, data} => {
                w.write_all(data)
            }
        }
    }

    pub(crate) fn change_type(&self) -> u64 {
        match self {
            Change::AddNode {id: _, parent: _, index_in_parent: _} => ChangeType::ADD_NODE,
            Change::MoveNode {id: _, new_parent: _, index_in_new_parent: _} => ChangeType::MOVE_NODE,
            Change::DeleteNode {id: _} => ChangeType::REMOVE_NODE,
            Change::SetString {node: _, attribute: _, value: _} => ChangeType::SET_STRING,
            Change::SetBool {node: _, attribute: _, value: _} => ChangeType::SET_BOOL,
            Change::UnknownChange {change_type, data: _} => *change_type,
        }
    }

    pub fn combine_change(&self, last_change: &Change) -> Option<Change> {
        if let Change::SetString {node, attribute, value} = self {
            if let Change::SetString {node: node2, attribute: attribute2, value: _value2} = last_change {
                if node == node2 && attribute == attribute2 {
                    return Some(Change::SetString {node: node.clone(), attribute: attribute.clone(), value: value.clone()});
                }
            }
        }

        if let Change::SetBool {node, attribute, value} = self {
            if let Change::SetBool {node: node2, attribute: attribute2, value: _value2} = last_change {
                if node == node2 && attribute == attribute2 {
                    return Some(Change::SetBool {node: node.clone(), attribute: attribute.clone(), value: *value});
                }
            }
        }

        None
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::AddNode {id, parent, index_in_parent} => write!(f, "AddNode({} in {}[{}])", id, parent, index_in_parent),
            Change::MoveNode {id, new_parent, index_in_new_parent} => write!(f, "MoveNode({} to {}[{}])", id, new_parent, index_in_new_parent),
            Change::DeleteNode {id} => write!(f, "RemoveNode({})", id),
            Change::SetString {node, attribute, value} => write!(f, "SetString({}, {} = {})", node, attribute, value),
            Change::SetBool {node, attribute, value} => write!(f, "SetBool({}, {} = {})", node, attribute, value),
            Change::UnknownChange {change_type, data} => write!(f, "UnknownChange({}, {} bytes)", change_type, data.len()),
        }
    }
}