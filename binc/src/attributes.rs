

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{}", s),
            AttributeValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub fn attribute_type(value: &AttributeValue) -> &str {
    match value {
        AttributeValue::String(_) => "String",
        AttributeValue::Bool(_) => "Bool",
    }
}


#[derive(Debug, Clone, Default)]
pub struct AttributeStore {
    attributes: Vec<AttributeEntry>
}

#[derive(Debug, Clone)]
pub struct AttributeEntry {
    pub key: String,
    pub value: AttributeValue
}

impl AttributeStore {
    pub fn set(&mut self, key: &str, value: AttributeValue) {
        for a in &mut self.attributes {
            if a.key == key {
                a.value = value;
                return;
            }
        }

        self.attributes.push(AttributeEntry { key: key.to_string(), value });
    }

    pub fn get(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.iter().find(|x| x.key == key).map(|x| &x.value)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue> {
        self.attributes.iter_mut().find(|x| x.key == key).map(|x| &mut x.value)
    }

    pub fn iter(&self) -> std::slice::Iter<AttributeEntry> {
        self.attributes.iter()
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }
}