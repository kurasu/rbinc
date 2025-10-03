use crate::attributes::{attribute_type, AttributeValue};
use crate::node_id::NodeId;
use crate::node_store::NodeStore;
use crate::readwrite::{ReadExt, WriteExt};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::{Read, Write};

pub(crate) struct OperationIds;

#[allow(dead_code)]
impl OperationIds {
    // v1
    pub const ADD_NODE: u64 = 0x01;
    pub const REMOVE_NODE: u64 = 0x02;
    pub const MOVE_NODE: u64 = 0x03;
    pub const SET_TYPE: u64 = 0x04;
    pub const DEFINE_TYPE_NAME: u64 = 0x05;
    pub const SET_NAME: u64 = 0x06;
    pub const DEFINE_ATTRIBUTE_NAME: u64 = 0x07;
    pub const SET_BOOL: u64 = 0x08;
    pub const SET_STRING: u64 = 0x09;

    // work in progress
    pub const DEFINE_TAG_NAME: u64 = 0x14;

    pub const SNAPSHOT: u64 = 0x10;
    pub const CHECKSUM: u64 = 0x11;

    pub const ADD_TAG: u64 = 0x18;
    pub const REMOVE_TAG: u64 = 0x19;

    pub const ADD_SOURCE: u64 = 0x21;
    pub const UPDATE_SOURCE: u64 = 0x22;
    pub const REMOVE_SOURCE: u64 = 0x23;

    pub const ADD_COMMENT: u64 = 0x31;

    pub const SET_UUID: u64 = 0x42;
    pub const SET_UINT8: u64 = 0x43;
    pub const SET_UINT16: u64 = 0x44;
    pub const SET_UINT24: u64 = 0x45;
    pub const SET_UINT32: u64 = 0x46;
    pub const SET_UINT64: u64 = 0x47;
    pub const SET_INT8: u64 = 0x48;
    pub const SET_INT16: u64 = 0x49;
    pub const SET_INT24: u64 = 0x4A;
    pub const SET_INT32: u64 = 0x4B;
    pub const SET_INT64: u64 = 0x4C;
    pub const SET_FLOAT16: u64 = 0x4D;
    pub const SET_FLOAT32: u64 = 0x4E;
    pub const SET_FLOAT64: u64 = 0x4F;

    pub const SET_BOOL_ARRAY: u64 = 0x60;
    pub const SET_STRING_ARRAY: u64 = 0x61;
    pub const SET_UUID_ARRAY: u64 = 0x62;
    pub const SET_UINT8_ARRAY: u64 = 0x63;
    pub const SET_UINT16_ARRAY: u64 = 0x64;
    pub const SET_UINT24_ARRAY: u64 = 0x65;
    pub const SET_UINT32_ARRAY: u64 = 0x66;
    pub const SET_UINT64_ARRAY: u64 = 0x67;
    pub const SET_INT8_ARRAY: u64 = 0x68;
    pub const SET_INT16_ARRAY: u64 = 0x69;
    pub const SET_INT24_ARRAY: u64 = 0x6A;
    pub const SET_INT32_ARRAY: u64 = 0x6B;
    pub const SET_INT64_ARRAY: u64 = 0x6C;
    pub const SET_FLOAT16_ARRAY: u64 = 0x6D;
    pub const SET_FLOAT32_ARRAY: u64 = 0x6E;
    pub const SET_FLOAT64_ARRAY: u64 = 0x6F;
}

#[derive(Debug, Clone)]
pub enum Operation {
    /// Add a new node to the document tree with a given parent and index
    AddNode {
        id: NodeId,
        node_type: usize,
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

    /// Defines a user-readable name for a type
    DefineTypeName { id: usize, name: String },

    /// Set the name of a node
    SetName { node: NodeId, name: String },

    /// Defines a user-readable name for an attribute id
    DefineAttributeName { id: usize, name: String },

    /// Set an attribute on a node
    SetAttribute {
        node: NodeId,
        attribute: usize,
        value: AttributeValue,
    },

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

    /// Add a comment to a node
    AddComment {
        node: NodeId,
        comment: String,
        author: String,
        response_to: usize,
    },

    /// Unknown change type. Since the size is known, the data can be read and written without knowing the type
    UnknownOperation { operation: u64, data: Vec<u8> },
}

impl Operation {
    // This is an id used to locate checksums in the file. In case of corruption this can be used to
    // locate which ranges of the file are corrupted and automatically repair them using other sources.
    pub const HASH_ID: u32 = u32::from_be_bytes(*b"h@sH");

    pub(crate) fn apply(&self, nodes: &mut NodeStore) {
        match self {
            Operation::AddNode {
                id,
                node_type,
                parent,
                index_in_parent,
            } => {
                nodes.add(*id, *node_type, *parent, *index_in_parent as usize);
            }
            Operation::RemoveNode { id } => {
                nodes.delete_recursive(*id);
            }
            Operation::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => {
                nodes.move_node(*id, *new_parent, *index_in_new_parent as usize);
            }
            Operation::SetType { node, type_id: id } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_type(*id);
            }
            Operation::SetName { node, name } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_name(name);
            }
            Operation::DefineTypeName { id, name } => {
                nodes.define_type_name(*id, name);
            }
            Operation::DefineAttributeName { id, name } => {
                nodes.define_attribute_name(*id, name);
            }
            Operation::DefineTagName { id, name } => {
                nodes.define_tag_name(*id, name);
            }
            Operation::SetTag { node, tag } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_tag(*tag);
            }
            Operation::RemoveTag { node, tag } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.clear_tag(*tag);
            }
            Operation::Snapshot {
                author: _,
                message: _,
            } => {
                // no-op
            }
            Operation::Checksum { data: _ } => {
                // no-op
            }
            Operation::SetAttribute {
                node,
                attribute,
                value,
            } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.set_attribute(*attribute, value.clone());
            }
            Operation::AddComment {
                node,
                comment,
                author,
                response_to,
            } => {
                let x = nodes.get_mut(*node).expect("Node not found");
                x.add_comment(comment, author, *response_to);
            }
            Operation::UnknownOperation {
                operation: _,
                data: _,
            } => {
                // Do nothing
            }
        }
    }

    pub(crate) fn read<T: Read>(r: &mut T) -> io::Result<Operation> {
        let operation = r.read_length_flipped()? as u64;
        let size = r.read_length()?;
        match operation {
            OperationIds::ADD_NODE => {
                let id = r.read_id()?;
                let node_type = r.read_length()?;
                let parent = r.read_id()?;
                let index_in_parent = r.read_length()?;
                Ok(Operation::AddNode {
                    id,
                    node_type,
                    parent,
                    index_in_parent,
                })
            }
            OperationIds::REMOVE_NODE => {
                let node = r.read_id()?;
                Ok(Operation::RemoveNode { id: node })
            }
            OperationIds::MOVE_NODE => {
                let id = r.read_id()?;
                let new_parent = r.read_id()?;
                let index_in_new_parent = r.read_length()?;
                Ok(Operation::MoveNode {
                    id,
                    new_parent,
                    index_in_new_parent,
                })
            }
            OperationIds::SNAPSHOT => {
                let author = r.read_string()?;
                let message = r.read_string()?;
                Ok(Operation::Snapshot { author, message })
            }
            OperationIds::CHECKSUM => {
                let hash = r.read_u32()?;
                if hash != Operation::HASH_ID {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid hash id {}", hash),
                    ));
                }
                let data = r.read_bytes()?;
                Ok(Operation::Checksum { data })
            }
            OperationIds::SET_STRING => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_string()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::String(value),
                })
            }
            OperationIds::SET_BOOL => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u8()? != 0;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::Bool(value),
                })
            }
            OperationIds::SET_UUID => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_uuid()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::Uuid(value),
                })
            }
            OperationIds::SET_UINT8 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u8()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U8(value),
                })
            }
            OperationIds::SET_UINT16 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u16()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U16(value),
                })
            }
            OperationIds::SET_UINT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u32()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U32(value),
                })
            }
            OperationIds::SET_UINT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_u64()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::U64(value),
                })
            }
            OperationIds::SET_INT8 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i8()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I8(value),
                })
            }
            OperationIds::SET_INT16 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i16()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I16(value),
                })
            }
            OperationIds::SET_INT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i32()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I32(value),
                })
            }
            OperationIds::SET_INT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_i64()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::I64(value),
                })
            }
            OperationIds::SET_FLOAT32 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_f32()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::F32(value),
                })
            }
            OperationIds::SET_FLOAT64 => {
                let node = r.read_id()?;
                let attribute = r.read_length()?;
                let value = r.read_f64()?;
                Ok(Operation::SetAttribute {
                    node,
                    attribute,
                    value: AttributeValue::F64(value),
                })
            }
            OperationIds::SET_NAME => {
                let node = r.read_id()?;
                let name = r.read_string()?;
                Ok(Operation::SetName { node, name })
            }
            OperationIds::SET_TYPE => {
                let node = r.read_id()?;
                let type_id = r.read_length()?;
                Ok(Operation::SetType {
                    node,
                    type_id: type_id,
                })
            }
            OperationIds::DEFINE_TYPE_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Operation::DefineTypeName { id, name })
            }
            OperationIds::DEFINE_ATTRIBUTE_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Operation::DefineAttributeName { id, name })
            }
            OperationIds::DEFINE_TAG_NAME => {
                let id = r.read_length()?;
                let name = r.read_string()?;
                Ok(Operation::DefineTagName { id, name })
            }
            OperationIds::ADD_COMMENT => {
                let node = r.read_id()?;
                let comment = r.read_string()?;
                let author = r.read_string()?;
                let response_to = r.read_length()?;
                Ok(Operation::AddComment {
                    node,
                    comment,
                    author,
                    response_to,
                })
            }
            _ => {
                let mut data = vec![0; size as usize];
                r.read_exact(&mut data)?;
                Ok(Operation::UnknownOperation { operation, data })
            }
        }
    }

    pub fn write<T: Write>(&self, w: &mut T) -> io::Result<()> {
        let mut temp: Vec<u8> = vec![];
        self.write_content(&mut temp)?;

        // header (id+size)
        w.write_length_flipped(self.operation_id() as usize)?; // Flipped for better resiliency
        w.write_length(temp.len())?;

        // content
        w.write_all(&temp)
    }

    fn write_content<T: Write>(&self, w: &mut T) -> io::Result<()> {
        match self {
            Operation::AddNode {
                id,
                node_type,
                parent,
                index_in_parent,
            } => {
                w.write_id(id)?;
                w.write_length(*node_type)?;
                w.write_id(parent)?;
                w.write_length(*index_in_parent)
            }
            Operation::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => {
                w.write_id(id)?;
                w.write_id(new_parent)?;
                w.write_length(*index_in_new_parent)
            }
            Operation::RemoveNode { id } => w.write_id(id),
            Operation::Snapshot { author, message } => {
                w.write_string(author)?;
                w.write_string(message)
            }
            Operation::Checksum { data } => {
                w.write_u32(Operation::HASH_ID)?;
                w.write_bytes(data)
            }
            Operation::SetName { node, name: label } => {
                w.write_id(node)?;
                w.write_string(label)
            }
            Operation::SetType { node, type_id } => {
                w.write_id(node)?;
                w.write_length(*type_id)
            }
            Operation::DefineTypeName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Operation::DefineAttributeName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Operation::DefineTagName { id, name } => {
                w.write_length(*id)?;
                w.write_string(name)
            }
            Operation::SetTag { node, tag } => {
                w.write_id(node)?;
                w.write_length(*tag)
            }
            Operation::RemoveTag { node, tag } => {
                w.write_id(node)?;
                w.write_length(*tag)
            }
            Operation::SetAttribute {
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
                    AttributeValue::U24(u) => w.write_bytes(u),
                    AttributeValue::U32(u) => w.write_u32(*u),
                    AttributeValue::U64(u) => w.write_u64(*u),
                    AttributeValue::I8(u) => w.write_i8(*u),
                    AttributeValue::I16(u) => w.write_i16(*u),
                    AttributeValue::I24(u) => w.write_bytes(u),
                    AttributeValue::I32(u) => w.write_i32(*u),
                    AttributeValue::I64(u) => w.write_i64(*u),
                    AttributeValue::F32(u) => w.write_f32(*u),
                    AttributeValue::F64(u) => w.write_f64(*u),
                }
            }
            Operation::AddComment {
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
            Operation::UnknownOperation { operation: _, data } => w.write_all(data),
        }
    }

    pub(crate) fn operation_id(&self) -> u64 {
        match self {
            Operation::AddNode {
                id: _,
                node_type: _,
                parent: _,
                index_in_parent: _,
            } => OperationIds::ADD_NODE,
            Operation::MoveNode {
                id: _,
                new_parent: _,
                index_in_new_parent: _,
            } => OperationIds::MOVE_NODE,
            Operation::RemoveNode { id: _ } => OperationIds::REMOVE_NODE,
            Operation::Snapshot {
                author: _,
                message: _,
            } => OperationIds::SNAPSHOT,
            Operation::Checksum { data: _ } => OperationIds::CHECKSUM,
            Operation::SetName { node: _, name: _ } => OperationIds::SET_NAME,
            Operation::SetType {
                node: _,
                type_id: _,
            } => OperationIds::SET_TYPE,
            Operation::DefineTypeName { id: _, name: _ } => OperationIds::DEFINE_TYPE_NAME,
            Operation::DefineAttributeName { id: _, name: _ } => {
                OperationIds::DEFINE_ATTRIBUTE_NAME
            }
            Operation::DefineTagName { id: _, name: _ } => OperationIds::DEFINE_TAG_NAME,
            Operation::SetTag { node: _, tag: _ } => OperationIds::ADD_TAG,
            Operation::RemoveTag { node: _, tag: _ } => OperationIds::REMOVE_TAG,
            Operation::SetAttribute {
                node: _,
                attribute: _,
                value,
            } => match value {
                AttributeValue::String(_) => OperationIds::SET_STRING,
                AttributeValue::Bool(_) => OperationIds::SET_BOOL,
                AttributeValue::Uuid(_) => OperationIds::SET_UUID,
                AttributeValue::U8(_) => OperationIds::SET_UINT8,
                AttributeValue::U16(_) => OperationIds::SET_UINT16,
                AttributeValue::U24(_) => OperationIds::SET_UINT24,
                AttributeValue::U32(_) => OperationIds::SET_UINT32,
                AttributeValue::U64(_) => OperationIds::SET_UINT64,
                AttributeValue::I8(_) => OperationIds::SET_INT8,
                AttributeValue::I16(_) => OperationIds::SET_INT16,
                AttributeValue::I24(_) => OperationIds::SET_INT24,
                AttributeValue::I32(_) => OperationIds::SET_INT32,
                AttributeValue::I64(_) => OperationIds::SET_INT64,
                AttributeValue::F32(_) => OperationIds::SET_FLOAT32,
                AttributeValue::F64(_) => OperationIds::SET_FLOAT64,
            },
            Operation::AddComment {
                node: _,
                comment: _,
                author: _,
                response_to: _,
            } => OperationIds::ADD_COMMENT,
            Operation::UnknownOperation { operation, data: _ } => *operation,
        }
    }

    pub fn combine_operations(&self, previous_operation: &Operation) -> Option<Operation> {
        if let Operation::SetAttribute {
            node,
            attribute,
            value,
        } = self
        {
            if let Operation::SetAttribute {
                node: node2,
                attribute: attribute2,
                value: _value2,
            } = previous_operation
            {
                if node == node2 && attribute == attribute2 {
                    return Some(Operation::SetAttribute {
                        node: *node,
                        attribute: *attribute,
                        value: value.clone(),
                    });
                }
            }
        }

        if let Operation::SetName { node, name: label } = self {
            if let Operation::SetName {
                node: node2,
                name: _label2,
            } = previous_operation
            {
                if node == node2 {
                    return Some(Operation::SetName {
                        node: node.clone(),
                        name: label.clone(),
                    });
                }
            }
        }

        None
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::AddNode {
                id,
                node_type,
                parent,
                index_in_parent,
            } => write!(
                f,
                "AddNode({}[{}] in {}[{}])",
                id, node_type, parent, index_in_parent
            ),
            Operation::MoveNode {
                id,
                new_parent,
                index_in_new_parent,
            } => write!(
                f,
                "MoveNode({} to {}[{}])",
                id, new_parent, index_in_new_parent
            ),
            Operation::RemoveNode { id } => write!(f, "RemoveNode({})", id),
            Operation::Snapshot { author, message } => {
                write!(f, "Snapshot by {} ({})", author, message)
            }
            Operation::Checksum { data } => write!(f, "Checksum({} bytes)", data.len()),
            Operation::SetType { node, type_id } => write!(f, "SetType({}, {})", node, type_id),
            Operation::SetName { node, name: label } => write!(f, "SetLabel({}, {})", node, label),
            Operation::DefineTypeName { id, name } => write!(f, "SetTypeName({}, {})", id, name),
            Operation::DefineAttributeName { id, name } => {
                write!(f, "SetAttributeName({}, {})", id, name)
            }
            Operation::DefineTagName { id, name } => write!(f, "SetTagName({}, {})", id, name),
            Operation::SetTag { node, tag } => write!(f, "SetTag({}, {})", node, tag),
            Operation::RemoveTag { node, tag } => write!(f, "RemoveTag({}, {})", node, tag),
            Operation::SetAttribute {
                node,
                attribute,
                value,
            } => {
                if value.too_long_for_display() {
                    write!(
                        f,
                        "Set{}({}, {} = {})",
                        attribute_type(value),
                        node,
                        attribute,
                        "<...>"
                    )
                } else {
                    write!(
                        f,
                        "Set{}({}, {} = {})",
                        attribute_type(value),
                        node,
                        attribute,
                        value
                    )
                }
            }
            Operation::UnknownOperation {
                operation: change_type,
                data,
            } => {
                write!(f, "UnknownChange({}, {} bytes)", change_type, data.len())
            }
            Operation::AddComment {
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
