use crate::document::Node;

#[derive(Default)]
pub struct Tree {
    /// Root node of the tree
    pub root: Node,
}

impl Tree {
    /// Get the parent_node along with the name of the node
    pub fn get_parent_mut(&mut self, path: &String) -> (Option<&mut Node>, String) {
        if path.starts_with("/") {
            return (Some(&mut self.root), path[1..].to_string());
        }

        let parts = path.split("/");
        let name = parts.clone().last().unwrap().clone().to_string();
        
        if parts.count() == 1 {
            return (Some(&mut self.root), name);
        }
        
        let parent_path = &path[..path.len() - name.len() - 1].to_string();
        let parent = self.get_mut(parent_path);
        (parent, name)
    }

    fn get_recursive<'a>(node: &'a Node, parts: &mut std::str::Split<'_, &str>) -> Option<&'a Node> {
        if let Some(part) = parts.next() {
            if let Some(child) = node.children.iter().find(|x| x.name == part) {
                return Self::get_recursive(child, parts);
            } else {
                return None;
            }
        }
        Some(node)
    }

    pub fn get(&self, path: &str) -> Option<&Node> {

        if path.starts_with("/") {
            return Self::get_recursive(&self.root, &mut path[1..].split("/"));
        }
        
        if path.is_empty() {
            return Some(&self.root);
        }

        let mut parts = path.split("/");
        Self::get_recursive(&self.root, &mut parts)
    }

    fn get_mut_recursive<'a>(node: &'a mut Node, parts: &mut std::str::Split<'_, &str>) -> Option<&'a mut Node> {
        if let Some(part) = parts.next() {
            if let Some(child) = node.children.iter_mut().find(|x| x.name == part) {
                return Self::get_mut_recursive(child, parts);
            } else {
                return None;
            }
        }
        Some(node)
    }

    pub fn get_mut(&mut self, path: &String) -> Option<&mut Node> {
        if path.starts_with("/") {
            return Self::get_mut_recursive(&mut self.root, &mut path[1..].split("/"));
        }
        
        if path.is_empty() {
            return Some(&mut self.root);
        }

        let mut parts = path.split("/");
        Self::get_mut_recursive(&mut self.root, &mut parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Node;

    fn create_test_tree() -> Tree {
        let mut tree = Tree::default();
        let mut child1 = Node::new("child1");
        let child2 = Node::new("child2");
        let sub = Node::new("sub");
        child1.children.push(sub);
        tree.root.children.push(child1);
        tree.root.children.push(child2);
        tree
    }

    #[test]
    fn test_get_node_child1() {
        let tree = create_test_tree();
        let node = tree.get("child1");
        assert!(node.is_some());
        assert_eq!(node.unwrap().name, "child1");
        let node2 = tree.get("/child1");
        assert!(node2.is_some());
        assert_eq!(node2.unwrap().name, "child1");
    }

    #[test]
    fn test_get_node_recursive() {
        let mut tree = Tree::default();
        let mut child1 = Node::new("1");
        let mut child2 = Node::new("2");
        let mut child3 = Node::new("3");
        let mut child4 = Node::new("4");

        child3.children.push(child4);
        child2.children.push(child3);
        child1.children.push(child2);
        tree.root.children.push(child1);
        tree.root.name = "root".to_string();

        let node1 = tree.get("1");
        assert!(node1.is_some());
        assert_eq!(node1.unwrap().name, "1");
        let node2 = tree.get("1/2");
        assert!(node2.is_some());
        assert_eq!(node2.unwrap().name, "2");
        let node3 = tree.get_mut(&"1/2/3".to_string());
        assert!(node3.is_some());
        assert_eq!(node3.unwrap().name, "3");
        let node4 = tree.get("/1/2/3/4");
        assert!(node4.is_some());
        assert_eq!(node4.unwrap().name, "4");

        let (parent1, name1) = tree.get_parent_mut(&"1/3".to_string());
        assert!(parent1.is_some());
        assert_eq!(parent1.unwrap().name, "1");
        assert_eq!(name1, "3");

        let (parent2, name2) = tree.get_parent_mut(&"/a".to_string());
        assert!(parent2.is_some());
        assert_eq!(parent2.unwrap().name, "root");
        assert_eq!(name2, "a");
    }

    #[test]
    fn test_get_node_child1_sub() {
        let mut tree = create_test_tree();
        let node = tree.get("child1/sub");
        assert!(node.is_some());
        assert_eq!(node.unwrap().name, "sub");

        let (parent, name) = tree.get_parent_mut(&"child1/sub".to_string());
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().name, "child1");
        assert_eq!(name, "sub");
    }

    #[test]
    fn test_get_node_nonexistent() {
        let tree = create_test_tree();
        let node = tree.get("nonexistent");
        assert!(node.is_none());
    }
}
