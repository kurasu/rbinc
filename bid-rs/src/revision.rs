use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::io::ReadExt;
use crate::io::WriteExt;

enum ChangeType {
    AddNode = 0x01,
    AddNodeFromSource = 0x02,
    RemoveNode = 0x08,

    AddChild = 0x11,
    RemoveChild = 0x12,

    AddSource = 0x21,
    UpdateSource = 0x22,
    RemoveSource = 0x23,

    AddComment = 0x31,

    SetBool = 0x40,
    SetString = 0x41,
    SetUUID = 0x42,
    SetUInt8 = 0x43,
    SetUInt16 = 0x44,
    SetUInt32 = 0x45,
    SetUInt64 = 0x46,
    SetInt8 = 0x47,
    SetInt16 = 0x48,
    SetInt32 = 0x49,
    SetInt64 = 0x4A,
    SetFloat16 = 0x4B,
    SetFloat32 = 0x4C,
    SetFloat64 = 0x4D,
    SetFloat80 = 0x4E,

    SetBoolArray = 0x60,
    SetStringArray = 0x61,
    SetUUIDArray = 0x62,
    SetUInt8Array = 0x63,
    SetUInt16Array = 0x64,
    SetUInt32Array = 0x65,
    SetUInt64Array = 0x66,
    SetInt8Array = 0x67,
    SetInt16Array = 0x68,
    SetInt32Array = 0x69,
    SetInt64Array = 0x6A,
    SetFloat16Array = 0x6B,
    SetFloat32Array = 0x6C,
    SetFloat64Array = 0x6D,
    SetFloat80Array = 0x6E,

    Unknown = 0x7FFFFE, // Only used internally
    Error = 0x7FFFFF, // Only used internally
}

pub trait Change {
    fn change_type(&self) -> ChangeType;
    fn write(&self, w: &mut dyn Write) -> io::Result<()>;
}

struct AddNode {
    node: Uuid
}

impl AddNode {
    pub fn new(node: Uuid) -> Self {
        Self { node }
    }

    pub fn read(r: &mut dyn Read) -> io::Result<Self> {
        let node = r.read_uuid()?;
        Ok(Self { node })
    }
}

impl Change for AddNode {
    fn change_type(&self) -> ChangeType { ChangeType::AddNode }
    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_length(self.change_type() as u64)?;
        w.write_uuid(self.node)
    }
}
