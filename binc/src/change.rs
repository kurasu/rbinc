use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use crate::document::{Document, Node};
use crate::iowrappers::{ReadExt, WriteExt};
use crate::tree::Tree;

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
    AddNode {path: String},
    RemoveNode {path: String},
    SetString {path: String, attribute: String, value: String},
    SetBool {path: String, attribute: String, value: bool},
    UnknownChange {change_type: u64, data: Vec<u8>},
}

impl Change {
    pub(crate) fn apply(&self, tree: &mut Tree) -> io::Result<()>
    {
        match self {
            Change::AddNode {path} => {
                let (parent, name) = tree.get_parent_mut(path);
                if let Some(parent) = parent {
                    parent.children.push(Node::new(&name));
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Parent not found"))
                }
            }
            Change::RemoveNode { path } => {
                let (parent, name) = tree.get_parent_mut(path);
                if let Some(parent) = parent {
                    let index = parent.children.iter().position(|x| x.name == name).ok_or(io::Error::new(io::ErrorKind::NotFound, "Node not found"))?;
                    parent.children.remove(index);
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Parent not found"))
                }
            }
            Change::SetString {path, attribute, value} => {
                let x = tree.get_mut(path).expect("Node not found");
                x.set_string_attribute(attribute, value);
                Ok(())
            }
            Change::SetBool {path, attribute, value} => {
                let x = tree.get_mut(path).expect("Node not found");
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
                let path = r.read_string()?;
                Ok(Change::AddNode {path })
            }
            ChangeType::REMOVE_NODE => {
                let path = r.read_string()?;
                Ok(Change::RemoveNode {path })
            }
            ChangeType::SET_STRING => {
                let path = r.read_string()?;
                let attribute = r.read_string()?;
                let value = r.read_string()?;
                Ok(Change::SetString {path, attribute, value})
            }
            ChangeType::SET_BOOL => {
                let path = r.read_string()?;
                let attribute = r.read_string()?;
                let value = r.read_u8()? != 0;
                Ok(Change::SetBool {path, attribute, value})
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
            Change::AddNode {path} => {
                w.write_string(path)
            }
            Change::RemoveNode {path} => {
                w.write_string(path)
            }
            Change::SetString {path, attribute, value} => {
                w.write_string(path)?;
                w.write_string(attribute)?;
                w.write_string(value)
            }
            Change::SetBool {path, attribute, value} => {
                w.write_string(path)?;
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
            Change::AddNode {path: _} => ChangeType::ADD_NODE,
            Change::RemoveNode {path: _} => ChangeType::REMOVE_NODE,
            Change::SetString {path: _, attribute: _, value: _} => ChangeType::SET_STRING,
            Change::SetBool {path: _, attribute: _, value: _} => ChangeType::SET_BOOL,
            Change::UnknownChange {change_type, data: _} => *change_type,
        }
    }

    pub fn combine_change(&self, last_change: &Change) -> Option<Change> {
        if let Change::SetString {path, attribute, value} = self {
            if let Change::SetString {path: path2, attribute: attribute2, value: _value2} = last_change {
                if path == path2 && attribute == attribute2 {
                    return Some(Change::SetString {path: path.clone(), attribute: attribute.clone(), value: value.clone()});
                }
            }
        }

        if let Change::SetBool {path, attribute, value} = self {
            if let Change::SetBool {path: path2, attribute: attribute2, value: _value2} = last_change {
                if path == path2 && attribute == attribute2 {
                    return Some(Change::SetBool {path: path.clone(), attribute: attribute.clone(), value: *value});
                }
            }
        }

        None
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::AddNode {path} => write!(f, "AddNode({})", path),
            Change::RemoveNode {path} => write!(f, "RemoveNode({})", path),
            Change::SetString {path, attribute, value} => write!(f, "SetString({}, {} = {})", path, attribute, value),
            Change::SetBool {path, attribute, value} => write!(f, "SetBool({}, {} = {})", path, attribute, value),
            Change::UnknownChange {change_type, data} => write!(f, "UnknownChange({}, {} bytes)", change_type, data.len()),
        }
    }
}