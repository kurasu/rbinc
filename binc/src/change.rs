use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::document::Node;
use crate::iowrappers::{ReadExt, WriteExt};
use crate::util::shorten_uuid;

pub(crate) struct ChangeType;

impl ChangeType {
    pub const ADD_NODE: u64 = 0x01;
    pub const ADD_NODE_FROM_SOURCE: u64 = 0x02;
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

pub enum Change {
    AddNode {uuid: Uuid},
    RemoveNode {uuid: Uuid},
    AddChild {parent: Uuid, child: Uuid, insertion_index: u64},
    RemoveChild {parent: Uuid, child: Uuid},
    SetString {node: Uuid, attribute: String, value: String},
    SetBool {node: Uuid, attribute: String, value: bool},
    UnknownChange {change_type: u64, data: Vec<u8>},
}

impl Change {
    pub(crate) fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()>
    {
        match self {
            Change::AddNode {uuid} => {
                let old = nodes.insert(*uuid, Node::new());
                if old.is_some() {
                    return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Node already exists"));
                }
                Ok(())
            }
            Change::RemoveNode { uuid } => {
                let v = nodes.remove(uuid);
                if v.is_none() {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "Node not found"));
                }
                Ok(())
            }
            Change::AddChild {parent, child, insertion_index} => {
                if !nodes.contains_key(child) {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Child node not found"))?;
                }
                let parent_node = nodes.get_mut(parent).ok_or(io::Error::new(io::ErrorKind::NotFound, "Parent node not found"))?;
                parent_node.children.insert(*insertion_index as usize, *child);
                Ok(())
            }
            Change::RemoveChild {parent, child} => {
                let parent_node = nodes.get_mut(parent).ok_or(io::Error::new(io::ErrorKind::NotFound, "Parent node not found"))?;
                let child_index = parent_node.children.iter().position(|x| *x == *child).ok_or(io::Error::new(io::ErrorKind::NotFound, "Child node not found"))?;
                parent_node.children.remove(child_index);
                Ok(())
            }
            Change::SetString {node, attribute, value} => {
                let x = nodes.get_mut(node).ok_or(io::Error::new(io::ErrorKind::NotFound, "Node not found"))?;
                x.set_string_attribute(attribute, value);
                Ok(())
            }
            Change::SetBool {node, attribute, value} => {
                let x = nodes.get_mut(node).ok_or(io::Error::new(io::ErrorKind::NotFound, "Node not found"))?;
                x.set_bool_attribute(attribute, *value);
                Ok(())
            }
            Change::UnknownChange {change_type: _, data: _} => {
                // Do nothing
                Ok(())
            }
        }
    }

    pub(crate) fn read(mut r: &mut dyn Read) -> io::Result<Change> {
        let change_type = r.read_length()?;
        let change_size = r.read_length()?;
        match change_type {
            ChangeType::ADD_NODE => {
                let node = r.read_uuid()?;
                Ok(Change::AddNode {uuid: node})
            }
            ChangeType::REMOVE_NODE => {
                let node = r.read_uuid()?;
                Ok(Change::RemoveNode {uuid: node})
            }
            ChangeType::SET_STRING => {
                let node = r.read_uuid()?;
                let attribute = r.read_string()?;
                let value = r.read_string()?;
                Ok(Change::SetString {node, attribute, value})
            }
            ChangeType::SET_BOOL => {
                let node = r.read_uuid()?;
                let attribute = r.read_string()?;
                let value = r.read_u8()? != 0;
                Ok(Change::SetBool {node, attribute, value})
            }
            ChangeType::ADD_CHILD => {
                let parent = r.read_uuid()?;
                let child = r.read_uuid()?;
                let insertion_index = r.read_length()?;
                Ok(Change::AddChild {parent, child, insertion_index})
            }
            ChangeType::REMOVE_CHILD => {
                let parent = r.read_uuid()?;
                let child = r.read_uuid()?;
                Ok(Change::RemoveChild {parent, child})
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
            Change::AddNode {uuid} => {
                w.write_uuid(uuid)
            }
            Change::RemoveNode {uuid} => {
                w.write_uuid(uuid)
            }
            Change::AddChild {parent, child, insertion_index} => {
                w.write_uuid(parent)?;
                w.write_uuid(child)?;
                w.write_length(*insertion_index)
            }
            Change::RemoveChild {parent, child} => {
                w.write_uuid(parent)?;
                w.write_uuid(child)
            }
            Change::SetString {node, attribute, value} => {
                w.write_uuid(node)?;
                w.write_string(attribute)?;
                w.write_string(value)
            }
            Change::SetBool {node, attribute, value} => {
                w.write_uuid(node)?;
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
            Change::AddNode {uuid: _} => ChangeType::ADD_NODE,
            Change::RemoveNode {uuid: _} => ChangeType::REMOVE_NODE,
            Change::AddChild {parent: _, child: _, insertion_index: _} => ChangeType::ADD_CHILD,
            Change::RemoveChild {parent: _, child: _} => ChangeType::REMOVE_CHILD,
            Change::SetString {node: _, attribute: _, value: _} => ChangeType::SET_STRING,
            Change::SetBool {node: _, attribute: _, value: _} => ChangeType::SET_BOOL,
            Change::UnknownChange {change_type, data: _} => *change_type,
        }
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::AddNode {uuid} => write!(f, "AddNode({})", shorten_uuid(uuid)),
            Change::RemoveNode {uuid} => write!(f, "RemoveNode({})", shorten_uuid(uuid)),
            Change::AddChild {parent, child, insertion_index} => write!(f, "AddChild({}, {}, {})", shorten_uuid(parent), shorten_uuid(child), insertion_index),
            Change::RemoveChild {parent, child} => write!(f, "RemoveChild({}, {})", shorten_uuid(parent), shorten_uuid(child)),
            Change::SetString {node, attribute, value} => write!(f, "SetString({}, {} = {})", shorten_uuid(node), attribute, value),
            Change::SetBool {node, attribute, value} => write!(f, "SetBool({}, {} = {})", shorten_uuid(node), attribute, value),
            Change::UnknownChange {change_type, data} => write!(f, "UnknownChange({}, {} bytes)", change_type, data.len()),
        }
    }
}