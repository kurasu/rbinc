use uuid::Uuid;

/*pub enum ChangeT {
    AddNode{node: Uuid},
    //AddNodeFromSource = 0x02,
    RemoveNode(Uuid),

    AddChild{node: Uuid, child: Uuid, index: u32},
    RemoveChild{node: Uuid, child: Uuid},

    //AddSource = 0x21,
    //UpdateSource = 0x22,
    //RemoveSource = 0x23,

    //AddComment = 0x31,

    SetBool{node: Uuid, field: String, value: bool},
    SetString{node: Uuid, field: String, value: String},
    SetUUID{node: Uuid, field: String, value: Uuid},
    SetUInt8{node: Uuid, field: String, value: u8},
    SetUInt16{node: Uuid, field: String, value: u16},
    SetUInt32{node: Uuid, field: String, value: u32},
    SetUInt64{node: Uuid, field: String, value: u64},
    SetInt8{node: Uuid, field: String, value: i8},
    SetInt16{node: Uuid, field: String, value: i16},
    SetInt32{node: Uuid, field: String, value: i32},
    SetInt64{node: Uuid, field: String, value: i64},
    //SetFloat16{node: Uuid, field: str, value: f16},
    SetFloat32{node: Uuid, field: String, value: f32},
    SetFloat64{node: Uuid, field: String, value: f64},
    //SetFloat80 = 0x4E,

    /*SetBoolArray{node: uuid, field: String, value: [bool]},
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
    //SetFloat16Array = 0x6B,
    SetFloat32Array = 0x6C,
    SetFloat64Array = 0x6D,
    //etFloat80Array = 0x6E,*/

    /*Unknown = 0x7FFFFE, // Only used internally
    Error = 0x7FFFFF, // Only used internally*/
}

pub fn get_change_id(change: Change) -> u64 {
    match change {
        Change::AddNode(_) => 0x01,
        Change::RemoveNode(_) => 0x02,
        Change::AddChild { .. } => 0x11,
        Change::RemoveChild { .. } => 0x12,
        Change::SetBool { .. } => 0x40,
        Change::SetString { .. } => 0x41,
        Change::SetUUID { .. } => 0x42,
        Change::SetUInt8 { .. } => 0x43,
        Change::SetUInt16 { .. } => 0x44,
        Change::SetUInt32 { .. } => 0x45,
        Change::SetUInt64 { .. } => 0x46,
        Change::SetInt8 { .. } => 0x47,
        Change::SetInt16 { .. } => 0x48,
        Change::SetInt32 { .. } => 0x49,
        Change::SetInt64 { .. } => 0x4A,
        Change::SetFloat32 { .. } => 0x4C,
        Change::SetFloat64 { .. } => 0x4D,
    }
}*/
/*
pub fn describe(change: Change) -> String {
    return change;
}
*/
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
}

struct AddNode {
    node: Uuid
}

impl Change for AddNode {
    fn change_type(&self) -> ChangeType { ChangeType::AddNode }
}