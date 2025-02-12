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

        let mut parts = path.split("/");
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
