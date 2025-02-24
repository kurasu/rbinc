struct NameDictionary {
    names: Vec<Option<String>>,
}

impl NameDictionary {
    pub fn new() -> NameDictionary {
        NameDictionary { names: vec![] }
    }

    pub fn insert(&mut self, name: String, index: usize) {
        if index >= self.names.len() {
            self.names.resize(index + 1, None);
        }
        self.names[index] = Some(name);
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.names.get(index).and_then(|x| x.as_deref())
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|x| x.as_deref() == Some(name))
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }
}
