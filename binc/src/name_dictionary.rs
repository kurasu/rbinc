use std::cmp::max;

#[derive(Default)]
pub struct NameDictionary {
    names: Vec<Option<String>>,
}

impl NameDictionary {
    pub(crate) fn get_or_create_index(&self, name: &str) -> (usize, bool) {
        match self.get_index(name) {
            Some(index) => (index, true),
            None => {
                let next_id = max(self.names.len(), 1);
                (next_id, false)
            }
        }
    }
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
