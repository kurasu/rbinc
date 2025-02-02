use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::document::Node;
use crate::iowrappers::{ReadExt, WriteExt};
use crate::revision::Change;

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

pub struct AddNode {
    pub (crate) uuid: Uuid
}

impl AddNode {
    pub fn new(node: Uuid) -> Self {
        Self { uuid: node }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let node = r.read_uuid()?;
        Ok(Self { uuid: node })
    }
}

impl Change for AddNode {
    fn change_type(&self) -> u64 { ChangeType::ADD_NODE }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.uuid)
    }

    fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        let old = nodes.insert(self.uuid, Node::new());
        if old.is_some() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Node already exists"));
        }
        Ok(())
    }


}

pub struct RemoveNode {
    node: Uuid
}

impl RemoveNode {
    pub fn new(node: Uuid) -> Self {
        Self { node }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let node = r.read_uuid()?;
        Ok(Self { node })
    }
}

impl Change for RemoveNode {
    fn change_type(&self) -> u64 { ChangeType::REMOVE_NODE }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.node)
    }

    fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        let v = nodes.remove(&self.node);
        if v.is_none() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Node not found"));
        }
        Ok(())
    }
}

pub struct AddChild {
    pub(crate) parent: Uuid,
    pub(crate) child: Uuid,
    pub(crate)  insertion_index: u64,
}

impl AddChild {
    pub fn new(parent: Uuid, child: Uuid, insertion_index: u64) -> Self {
        Self { parent, child, insertion_index }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let parent = r.read_uuid()?;
        let child = r.read_uuid()?;
        let insertion_index = r.read_length()?;
        Ok(Self { parent, child, insertion_index })
    }
}

impl Change for AddChild {
    fn change_type(&self) -> u64 { ChangeType::ADD_CHILD }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.parent)?;
        w.write_uuid(&self.child)?;
        w.write_length(self.insertion_index)
    }

    fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        if !nodes.contains_key(&self.child) {
            Err(io::Error::new(io::ErrorKind::NotFound, "Child node not found"))?;
        }
        let parent_node = nodes.get_mut(&self.parent).ok_or(io::Error::new(io::ErrorKind::NotFound, "Parent node not found"))?;
        parent_node.children.insert(self.insertion_index as usize, self.child);
        Ok(())
    }
}

pub struct RemoveChild {
    pub(crate) parent: Uuid,
    pub(crate) child: Uuid,
}

impl RemoveChild {
    pub fn new(parent: Uuid, child: Uuid) -> Self {
        Self { parent, child }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let parent = r.read_uuid()?;
        let child = r.read_uuid()?;
        Ok(Self { parent, child })
    }
}

impl Change for RemoveChild {
    fn change_type(&self) -> u64 { ChangeType::REMOVE_CHILD }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.parent)?;
        w.write_uuid(&self.child)
    }

    fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        let parent_node = nodes.get_mut(&self.parent).ok_or(io::Error::new(io::ErrorKind::NotFound, "Parent node not found"))?;
        parent_node.children.retain(|child| *child != self.child);
        Ok(())
    }
}

pub struct SetString {
    pub (crate) node: Uuid,
    pub (crate) attribute: String,
    pub (crate) value: String,
}

impl SetString {
    pub fn new(node: Uuid, attribute: &str, value: String) -> Self {
        Self { node, attribute: attribute.to_string(), value }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let node = r.read_uuid()?;
        let attribute = r.read_string()?;
        let value = r.read_string()?;
        Ok(Self { node, attribute, value })
    }
}

impl Change for SetString {
    fn change_type(&self) -> u64 { ChangeType::SET_STRING }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.node)?;
        w.write_string(&self.attribute)?;
        w.write_string(&self.value)
    }

 fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        let x = nodes.get_mut(&self.node).ok_or(io::Error::new(io::ErrorKind::NotFound, "Node not found"))?;
        x.set_string_attribute(&self.attribute, &self.value);
        Ok(())
    }
}

pub struct UnknownChange {
    pub (crate) change_type: u64,
    pub (crate) data: Vec<u8>
}

impl UnknownChange {
    fn read(r: &mut dyn Read, change_type: u64, change_size: u64) -> io::Result<UnknownChange> {
        let mut data = vec![0; change_size as usize];
        r.read_exact(&mut data)?;
        Ok(UnknownChange { change_type, data })
    }
}

impl Display for UnknownChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnknownChange({}, {} bytes)", self.change_type, self.data.len())
    }
}

impl Change for UnknownChange {
    fn change_type(&self) -> u64 { self.change_type }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.data)
    }

    fn apply(&self, _nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        // Do nothing
        Ok(())
    }
}
pub struct SetBool {
    pub(crate) node: Uuid,
    pub(crate) attribute: String,
    pub(crate) value: bool,
}

impl SetBool {
    pub fn new(node: Uuid, attribute: &str, value: bool) -> Self {
        Self { node, attribute: attribute.to_string(), value }
    }

    pub fn read(mut r: &mut dyn Read, _change_size: u64) -> io::Result<Self> {
        let node = r.read_uuid()?;
        let attribute = r.read_string()?;
        let value = r.read_u8()? != 0;
        Ok(Self { node, attribute, value })
    }
}

impl Change for SetBool {
    fn change_type(&self) -> u64 { ChangeType::SET_BOOL }

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uuid(&self.node)?;
        w.write_string(&self.attribute)?;
        w.write_u8(self.value as u8)
    }

    fn apply(&self, nodes: &mut HashMap<Uuid, Node>) -> io::Result<()> {
        let x = nodes.get_mut(&self.node).ok_or(io::Error::new(io::ErrorKind::NotFound, "Node not found"))?;
        x.set_bool_attribute(&self.attribute, self.value);
        Ok(())
    }
}

fn shorten_uuid(uuid: Uuid) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}

impl Display for AddNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddNode({})", shorten_uuid(self.uuid))
    }
}

impl Display for RemoveNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoveNode({})", shorten_uuid(self.node))
    }
}


impl Display for AddChild {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddChild({}, {}, {})", shorten_uuid(self.parent), shorten_uuid(self.child), self.insertion_index)
    }
}

impl Display for RemoveChild {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoveChild({}, {})", shorten_uuid(self.parent), shorten_uuid(self.child))
    }
}

impl Display for SetString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SetString({}, {} = {})", shorten_uuid(self.node), self.attribute, self.value)
    }
}

impl Display for SetBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SetBool({}, {} = {})", shorten_uuid(self.node), self.attribute, self.value)
    }
}

pub(crate) fn read_change(mut r: &mut dyn Read) -> io::Result<Box<dyn Change>> {
    let change_type = r.read_length()?;
    let change_size = r.read_length()?;
    match change_type {
        ChangeType::ADD_NODE => Ok(Box::new(AddNode::read(r, change_size)?)),
        ChangeType::REMOVE_NODE => Ok(Box::new(RemoveNode::read(r, change_size)?)),
        ChangeType::SET_STRING => Ok(Box::new(SetString::read(r, change_size)?)),
        ChangeType::SET_BOOL => Ok(Box::new(SetBool::read(r, change_size)?)),
        ChangeType::ADD_CHILD => Ok(Box::new(AddChild::read(r, change_size)?)),
        ChangeType::REMOVE_CHILD => Ok(Box::new(RemoveChild::read(r, change_size)?)),
        _ => Ok(Box::new(UnknownChange::read(r, change_type, change_size)?)),
    }
}