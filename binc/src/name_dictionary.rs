#[derive(Default)]
pub struct NameDictionary {
    names: Vec<Option<String>>,
}

impl NameDictionary {
    pub fn insert(&mut self, index: usize, name: &str) {
        if index >= self.names.len() {
            self.names.resize(index + 1, None);
        }
        self.names[index] = Some(name.to_string());
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
