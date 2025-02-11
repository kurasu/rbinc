

struct Path {
    string: String
}

impl Path {
    pub fn new(string: String) -> Path {
        Path { string }
    }

    pub fn get(&self) -> &str {
        &self.string
    }

    pub fn get_parts(&self) -> Vec<&str> {
        self.string.split('/').collect()
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.string.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let path = Path::new(String::from("/home/user"));
        assert_eq!(path.get(), "/home/user");
    }

    #[test]
    fn test_get_parts() {
        let path = Path::new(String::from("/home/user/docs"));
        let parts = path.get_parts();
        assert_eq!(parts, vec!["", "home", "user", "docs"]);
    }
}