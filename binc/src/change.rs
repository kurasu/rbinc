use crate::attributes::{attribute_type, AttributeValue};
use crate::node_id::NodeId;
use crate::node_store::NodeStore;
use crate::readwrite::{ReadExt, WriteExt};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::{Read, Write};

pub(crate) struct ChangeType;

#[allow(dead_code)]
impl ChangeType {
    pub const ADD_NODE: u64 = 0x01;
    pub const ADD_NODE_FROM_SOURCE: u64 = 0x02;
    pub const MOVE_NODE: u64 = 0x03;
    pub const REMOVE_NODE: u64 = 0x04;

    pub const SNAPSHOT: u64 = 0x8;
    pub const CHECKSUM: u64 = 0x9;

    pub const SET_TYPE: u64 = 0x10;
    pub const SET_NAME: u64 = 0x11;

    pub const DEFINE_TYPE_NAME: u64 = 0x12;
    pub const DEFINE_ATTRIBUTE_NAME: u64 = 0x13;
    pub const DEFINE_TAG_NAME: u64 = 0x14;

    pub const ADD_TAG: u64 = 0x18;
    pub const REMOVE_TAG: u64 = 0x19;

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
}

#[derive(Debug, Clone)]
pub enum Change {
    /// Add a new node to the document tree with a given parent and index
    AddNode {
        id: NodeId,
        parent: NodeId,
        index_in_parent: usize,
    },

    /// Move a node to a new parent
    MoveNode {
        id: NodeId,
        new_parent: NodeId,
        index_in_new_parent: usize,
    },

    /// Remove a node from the document tree
    RemoveNode { id: NodeId },

    /// Set the type-id for a node
    SetType { node: NodeId, type_id: usize },

    /// Set the name of a node
    SetName { node: NodeId, name: String },

    /// Defines a user-readable name for a type
    DefineTypeName { id: usize, name: String },

    /// Defines a user-readable name for an attribute id
    DefineAttributeName { id: usize, name: String },

    /// Defines a user-readable name for a tag id
    DefineTagName { id: usize, name: String },

    /// Set a tag on a node
    SetTag { node: NodeId, tag: usize },

    /// Remove a tag from a node
    RemoveTag { node: NodeId, tag: usize },

    /// Add a named snapshot of the document
    Snapshot { author: String, message: String },

    /// Add a checksum to the document up until this point. This can be used to verify the document is not corrupted
    Checksum { data: Vec<u8> },

    /// Set an attribute on a node
    SetAttribute {
        node: NodeId,
        attribute: usize,
        value: AttributeValue,
    },

    /// Add a comment to a node
    AddComment {
        node: NodeId,
        comment: String,
        author: String,
        response_to: usize,
    },

    /// Unknown change type. Since the size is known, the data can be read and written without knowing the type
    UnknownChange { change_type: u64, data: Vec<u8> },
}

impl Change {
    // This is an id used to locate checksums in the file. In case of corruption this can be used to
    // locate which ranges of the file are corrupted and automatically repair them using other sources.
    pub const HASH_ID: u32 = u32::from_be_bytes(*b"h@sH");

    pub(crate) fn apply(&self, nodes: &mut NodeStore) {
        match self {
            Change::AddNode {
                id,
                parent,
                index_in_parent,
            } => {
                nodes.add(*id, *parent, *index_in_parent as usize);
            }
            Change::RemoveNode { id } => {
                nodes.delete_recursive(*id);
            }
            Change::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => {
                nodes.move_node(*id, *new_parent, *index_in_new_parent as usize);
            }
            Change::SetType { node, type_id: id } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_type(*id);
            }
            Change::SetName { node, name } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_name(name);
            }
            Change::DefineTypeName { id, name } => {
                nodes.define_type_name(*id, name);
            }
            Change::DefineAttributeName { id, name } => {
                nodes.define_attribute_name(*id, name);
            }
            Change::DefineTagName { id, name } => {
                nodes.define_tag_name(*id, name);
            }
            Change::SetTag { node, tag } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_tag(*tag);
            }
            Change::RemoveTag { node, tag } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.clear_tag(*tag);
            }
            Change::Snapshot {
                author: _,
                message: _,
            } => {
                // no-op
            }
            Change::Checksum { data: _ } => {
                // no-op
            }
            Change::SetAttribute {
                node,
                attribute,
                value,
            } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_attribute(*attribute, value.clone());
            }
            Change::AddComment {
                node,
                comment,
                author,
                response_to,
            } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.add_comment(comment, author, *response_to);
            }
            Change::UnknownChange {
                change_type: _,
                data: _,
            } => {
                // Do nothing
            }
        }
    }

    pub(crate) fn read<T: Read>(r: &mut T) -> io::Result<Change> {
        let change_type = r.read_length_flipped()? as u64;
        let change_size = r.read_length()?;
        match change_type {
            ChangeType::ADD_NODE => {
                let id = r.read_id()?;
                let parent = r.read_id()?;
                let index_in_parent = r.read_length()?;
                Ok(Change::AddNode {
                    id,
                    parent,
                    index_in_parent,
                })
            }
            ChangeType::REMOVE_NODE => {
                let node = r.read_id()?;
                Ok(Change::RemoveNode { id: node })
            }
            ChangeType::MOVE_NODE => {
                let id = r.read_id()?;
                let new_parent = r.read_id()?;
                let index_in_new_parent = r.read_length()?;
                Ok(Change::MoveNode {
                    id,
                    new_parent,
                    index_in_new_parent,
                })
            }
            ChangeType::SNAPSHOT => {
                let author = r.read_string()?;
                let message = r.read_string()?;
                Ok(Change::Snapshot { author, message })
            }
            ChangeType::CHECKSUM => {
                let hash = r.read_u32()?;
                if hash != Change::HASH_ID {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid hash id {}", hash),
                    ));
                }
                let data = r.read_bytes()?;
                Ok(Change::Checksum { data })
            }
            ChangeType::SET_STRING => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_string()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::String(value),
                })
            }
            ChangeType::SET_BOOL => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u8()? != 0;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::Bool(value),
                })
            }
            ChangeType::SET_UUID => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_uuid()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::Uuid(value),
                })
            }
            ChangeType::SET_UINT8 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u8()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U8(value),
                })
            }
            ChangeType::SET_UINT16 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u16()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U16(value),
                })
            }
            ChangeType::SET_UINT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u32()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U32(value),
                })
            }
            ChangeType::SET_UINT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u64()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U64(value),
                })
            }
            ChangeType::SET_INT8 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i8()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I8(value),
                })
            }
            ChangeType::SET_INT16 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i16()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I16(value),
                })
            }
            ChangeType::SET_INT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i32()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I32(value),
                })
            }
            ChangeType::SET_INT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i64()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I64(value),
                })
            }
            ChangeType::SET_FLOAT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_f32()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::F32(value),
                })
            }
            ChangeType::SET_FLOAT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_f64()?;
                Ok(Change::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::F64(value),
                })
            }
            ChangeType::SET_NAME => {
                let node = r.read_id()?;
                let name = r.read_string()?;
                Ok(Change::SetName { node, name })
            }
            ChangeType::SET_TYPE => {
                let node = r.read_id()?;
                let type_id = r.read_length()?;
                Ok(Change::SetType {
                    node,
                    type_id: type_id,
                })
            }
            ChangeType::DEFINE_TYPE_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Change::DefineTypeName { id, name })
            }
            ChangeType::DEFINE_ATTRIBUTE_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Change::DefineAttributeName { id, name })
            }
            ChangeType::DEFINE_TAG_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Change::DefineTagName { id, name })
            }
            ChangeType::ADD_COMMENT => {
                let node = r.read_id()?;
                let comment = r.read_string()?;
                let author = r.read_string()?;
                let response_to = r.read_length()?;
                Ok(Change::AddComment {
                    node,
                    comment,
                    author,
                    response_to,
                })
            }
            _ => {
                let mut data = vec![0; change_size as usize];
                r.read_exact(&mut data)?;
                Ok(Change::UnknownChange { change_type, data })
            }
        }
    }

    pub fn write<T: Write>(&self, w: &mut T) -> io::Result<()> {
        let mut temp: Vec<u8> = vec![];
        self.write_content(&mut temp)?;

        // header (id+size)
        w.write_length_flipped(self.change_type() as usize)?; // Flipped for better resiliency
        w.write_length(temp.len())?;

        // content
        w.write_all(&temp)
    }

    fn write_content<T: Write>(&self, w: &mut T) -> io::Result<()> {
        match self {
            Change::AddNode {
                id,
                parent,
                index_in_parent,
            } => {
                w.write_id(id)?;
                w.write_id(parent)?;
                w.write_length(*index_in_parent)
            }
            Change::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => {
                w.write_id(id)?;
                w.write_id(new_parent)?;
                w.write_length(*index_in_new_parent)
            }
            Change::RemoveNode { id } => w.write_id(id),
            Change::Snapshot { author, message } => {
                w.write_string(author)?;
                w.write_string(message)
            }
            Change::Checksum { data } => {
                w.write_u32(Change::HASH_ID)?;
                w.write_bytes(data)
            }
            Change::SetName { node, name: label } => {
                w.write_id(node)?;
                w.write_string(label)
            }
            Change::SetType { node, type_id } => {
                w.write_id(node)?;
                w.write_length(*type_id)
            }
            Change::DefineTypeName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Change::DefineAttributeName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Change::DefineTagName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Change::SetTag { node, tag } => {
                w.write_id(node)?;
                w.write_length(*tag)
            }
            Change::RemoveTag { node, tag } => {
                w.write_id(node)?;
                w.write_length(*tag)
            }
            Change::SetAttribute {
                node,
                attribute,
                value,
            } => {
                w.write_id(node)?;
                w.write_length(*attribute)?;
                match value {
                    AttributeValue::String(s) => w.write_string(s),
                    AttributeValue::Bool(b) => w.write_u8(*b as u8),
                    AttributeValue::Uuid(u) => w.write_uuid(u),
                    AttributeValue::U8(u) => w.write_u8(*u),
                    AttributeValue::U16(u) => w.write_u16(*u),
                    AttributeValue::U32(u) => w.write_u32(*u),
                    AttributeValue::U64(u) => w.write_u64(*u),
                    AttributeValue::I8(u) => w.write_i8(*u),
                    AttributeValue::I16(u) => w.write_i16(*u),
                    AttributeValue::I32(u) => w.write_i32(*u),
                    AttributeValue::I64(u) => w.write_i64(*u),
                    AttributeValue::F32(u) => w.write_f32(*u),
                    AttributeValue::F64(u) => w.write_f64(*u),
                }
            }
            Change::AddComment {
                node,
                comment,
                author,
                response_to,
            } => {
                w.write_id(node)?;
                w.write_string(comment)?;
                w.write_string(author)?;
                w.write_length(*response_to)
            }
            Change::UnknownChange {
                change_type: _,
                data,
            } => w.write_all(data),
        }
    }

    pub(crate) fn change_type(&self) -> u64 {
        match self {
            Change::AddNode {
                id: _,
                parent: _,
                index_in_parent: _,
            } => ChangeType::ADD_NODE,
            Change::MoveNode {
                id: _,
                new_parent: _,
                index_in_new_parent: _,
            } => ChangeType::MOVE_NODE,
            Change::RemoveNode { id: _ } => ChangeType::REMOVE_NODE,
            Change::Snapshot {
                author: _,
                message: _,
            } => ChangeType::SNAPSHOT,
            Change::Checksum { data: _ } => ChangeType::CHECKSUM,
            Change::SetName { node: _, name: _ } => ChangeType::SET_NAME,
            Change::SetType {
                node: _,
                type_id: _,
            } => ChangeType::SET_TYPE,
            Change::DefineTypeName { id: _, name: _ } => ChangeType::DEFINE_TYPE_NAME,
            Change::DefineAttributeName { id: _, name: _ } => ChangeType::DEFINE_ATTRIBUTE_NAME,
            Change::DefineTagName { id: _, name: _ } => ChangeType::DEFINE_TAG_NAME,
            Change::SetTag { node: _, tag: _ } => ChangeType::ADD_TAG,
            Change::RemoveTag { node: _, tag: _ } => ChangeType::REMOVE_TAG,
            Change::SetAttribute {
                node: _,
                attribute: _,
                value,
            } => match value {
                AttributeValue::String(_) => ChangeType::SET_STRING,
                AttributeValue::Bool(_) => ChangeType::SET_BOOL,
                AttributeValue::Uuid(_) => ChangeType::SET_UUID,
                AttributeValue::U8(_) => ChangeType::SET_UINT8,
                AttributeValue::U16(_) => ChangeType::SET_UINT16,
                AttributeValue::U32(_) => ChangeType::SET_UINT32,
                AttributeValue::U64(_) => ChangeType::SET_UINT64,
                AttributeValue::I8(_) => ChangeType::SET_INT8,
                AttributeValue::I16(_) => ChangeType::SET_INT16,
                AttributeValue::I32(_) => ChangeType::SET_INT32,
                AttributeValue::I64(_) => ChangeType::SET_INT64,
                AttributeValue::F32(_) => ChangeType::SET_FLOAT32,
                AttributeValue::F64(_) => ChangeType::SET_FLOAT64,
            },
            Change::AddComment {
                node: _,
                comment: _,
                author: _,
                response_to: _,
            } => ChangeType::ADD_COMMENT,
            Change::UnknownChange {
                change_type,
                data: _,
            } => *change_type,
        }
    }

    pub fn combine_change(&self, last_change: &Change) -> Option<Change> {
        if let Change::SetAttribute {
            node,
            attribute,
            value,
        } = self
        {
            if let Change::SetAttribute {
                node: node2,
                attribute: attribute2,
                value: _value2,
            } = last_change
            {
                if node == node2 && attribute == attribute2 {
                    return Some(Change::SetAttribute {
                        node: *node,
                        attribute: *attribute,
                        value: value.clone(),
                    });
                }
            }
        }

        if let Change::SetName { node, name: label } = self {
            if let Change::SetName {
                node: node2,
                name: _label2,
            } = last_change
            {
                if node == node2 {
                    return Some(Change::SetName {
                        node: node.clone(),
                        name: label.clone(),
                    });
                }
            }
        }

        None
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::AddNode {
                id,
                parent,
                index_in_parent,
            } => write!(f, "AddNode({} in {}[{}])", id, parent, index_in_parent),
            Change::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => write!(
                f,
                "MoveNode({} to {}[{}])",
                id, new_parent, index_in_new_parent
            ),
            Change::RemoveNode { id } => write!(f, "RemoveNode({})", id),
            Change::Snapshot { author, message } => {
                write!(f, "Snapshot by {} ({})", author, message)
            }
            Change::Checksum { data } => write!(f, "Checksum({} bytes)", data.len()),
            Change::SetType { node, type_id } => write!(f, "SetType({}, {})", node, type_id),
            Change::SetName { node, name: label } => write!(f, "SetLabel({}, {})", node, label),
            Change::DefineTypeName { id, name } => write!(f, "SetTypeName({}, {})", id, name),
            Change::DefineAttributeName { id, name } => {
                write!(f, "SetAttributeName({}, {})", id, name)
            }
            Change::DefineTagName { id, name } => write!(f, "SetTagName({}, {})", id, name),
            Change::SetTag { node, tag } => write!(f, "SetTag({}, {})", node, tag),
            Change::RemoveTag { node, tag } => write!(f, "RemoveTag({}, {})", node, tag),
            Change::SetAttribute {
                node,
                attribute,
                value,
            } => if value.too_long_for_display() {
                write!(
                    f,
                    "Set{}({}, {} = {})",
                    attribute_type(value),
                    node,
                    attribute,
                    "<...>")
                } else {
                write!(
                    f,
                    "Set{}({}, {} = {})",
                    attribute_type(value),
                    node,
                    attribute,
                    value)

            },
            Change::UnknownChange { change_type, data } => {
                write!(f, "UnknownChange({}, {} bytes)", change_type, data.len())
            }
            Change::AddComment {
                node,
                comment,
                author,
                response_to,
            } => write!(
                f,
                "AddComment({}, {} by {} in response to {})",
                node, comment, author, response_to
            ),
        }
    }
}
