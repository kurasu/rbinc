use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::io::ReadExt;
use crate::io::WriteExt;

struct ChangeType;

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
    pub const SET_UUIDARRAY: u64 = 0x62;
    pub const SET_UINT8ARRAY: u64 = 0x63;
    pub const SET_UINT16ARRAY: u64 = 0x64;
    pub const SET_UINT32ARRAY: u64 = 0x65;
    pub const SET_UINT64ARRAY: u64 = 0x66;
    pub const SET_INT8ARRAY: u64 = 0x67;
    pub const SET_INT16ARRAY: u64 = 0x68;
    pub const SET_INT32ARRAY: u64 = 0x69;
    pub const SET_INT64ARRAY: u64 = 0x6A;
    pub const SET_FLOAT16ARRAY: u64 = 0x6B;
    pub const SET_FLOAT32ARRAY: u64 = 0x6C;
    pub const SET_FLOAT64ARRAY: u64 = 0x6D;
    pub const SET_FLOAT80ARRAY: u64 = 0x6E;

    pub const UNKNOWN: u64 = 0x7FFFFE; // Only used internally
    pub const ERROR: u64 = 0x7FFFFF; // Only used internally
}

pub trait Change {
    fn change_type(&self) -> u64;
    fn write(&self, w: &mut dyn Write) -> io::Result<()>;
}

struct AddNode {
    node: Uuid
}

impl AddNode {
    pub fn new(node: Uuid) -> Self {
        Self { node }
    }

    pub fn read(mut r: &mut dyn Read) -> io::Result<Self> {
        let node = r.read_uuid()?;
        Ok(Self { node })
    }
}

impl Change for AddNode {
    fn change_type(&self) -> u64 { ChangeType::ADD_NODE }
    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_length(self.change_type() as u64)?;
        w.write_uuid(&self.node)
    }
}

fn read_change(mut r: &mut dyn Read) -> io::Result<Box<dyn Change>> {
    let change_type = r.read_length()?;
    match change_type {
        ChangeType::ADD_NODE => Ok(Box::new(AddNode::read(r)?)),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

pub struct Revision {
    changes: Vec<Box<dyn Change>>,
    id: Uuid,
    uuid_of_parents: Vec<Uuid>,
    date: String,
    user_name: String,
    message: String,
    tags: Vec<String>,
}

impl Revision {

    pub const CHANGE_LIST_ID: u32 = 0x43686e67;

    pub(crate) fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_uint32(Self::CHANGE_LIST_ID)?;
        w.write_uuid(&self.id)?;
        w.write_uuid_array(&self.uuid_of_parents)?;
        w.write_string(&self.date)?;
        w.write_string(&self.user_name)?;
        w.write_string(&self.message)?;
        w.write_string_array(&self.tags)?;

        w.write_length(self.changes.len() as u64)?;
        for change in &self.changes {
            change.write(w)?;
        }
        Ok(())
    }
}
