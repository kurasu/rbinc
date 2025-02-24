#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Uuid(uuid::Uuid),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{}", s),
            AttributeValue::Bool(b) => write!(f, "{}", b),
            AttributeValue::Uuid(u) => write!(f, "{}", u),
            AttributeValue::U8(u) => write!(f, "{}", u),
            AttributeValue::U16(u) => write!(f, "{}", u),
            AttributeValue::U32(u) => write!(f, "{}", u),
            AttributeValue::U64(u) => write!(f, "{}", u),
            AttributeValue::I8(u) => write!(f, "{}", u),
            AttributeValue::I16(u) => write!(f, "{}", u),
            AttributeValue::I32(u) => write!(f, "{}", u),
            AttributeValue::I64(u) => write!(f, "{}", u),
            AttributeValue::F32(u) => write!(f, "{}", u),
            AttributeValue::F64(u) => write!(f, "{}", u),
        }
    }
}

pub fn attribute_type(value: &AttributeValue) -> &str {
    match value {
        AttributeValue::String(_) => "String",
        AttributeValue::Bool(_) => "Bool",
        AttributeValue::Uuid(_) => "Uuid",
        AttributeValue::U8(_) => "U8",
        AttributeValue::U16(_) => "U16",
        AttributeValue::U32(_) => "U32",
        AttributeValue::U64(_) => "U64",
        AttributeValue::I8(_) => "I8",
        AttributeValue::I16(_) => "I16",
        AttributeValue::I32(_) => "I32",
        AttributeValue::I64(_) => "I64",
        AttributeValue::F32(_) => "F32",
        AttributeValue::F64(_) => "F64",
    }
}

#[derive(Debug, Clone, Default)]
pub struct AttributeStore {
    attributes: Vec<AttributeEntry>,
}

#[derive(Debug, Clone)]
pub struct AttributeEntry {
    pub key: usize,
    pub value: AttributeValue,
}

impl AttributeStore {
    pub fn set(&mut self, key: usize, value: AttributeValue) {
        for a in &mut self.attributes {
            if a.key == key {
                a.value = value;
                return;
            }
        }

        self.attributes.push(AttributeEntry { key, value });
    }

    pub fn get(&self, key: usize) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|x| x.key == key)
            .map(|x| &x.value)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut AttributeValue> {
        self.attributes
            .iter_mut()
            .find(|x| x.key == key)
            .map(|x| &mut x.value)
    }

    pub fn iter(&self) -> std::slice::Iter<AttributeEntry> {
        self.attributes.iter()
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }
}
