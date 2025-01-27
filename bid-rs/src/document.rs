use std::collections::HashMap;
use uuid::Uuid;
use crate::repository::Repository;
use crate::revision::Change;

pub struct Node {
    children: Vec<Node>,
    attributes: HashMap<String, Box<()>>,
}

impl Node {
    pub(crate) fn
    new() -> Node {
        Node {
            children: vec![],
            attributes: HashMap::new(),
        }
    }
}

struct Document {
    repository: Repository,
    nodes: HashMap<Uuid, Node>,
}


fn compute_nodes(repository: &Repository) -> HashMap<Uuid, Node> {
    let mut nodes: HashMap<Uuid, Node> = HashMap::new();
    for rev in &repository.revisions {
        for change in &rev.changes {
            change.apply(&mut nodes);
        }
    }
    nodes
}

impl Document {
    pub fn new(repository: Repository) -> Document {
        let nodes = compute_nodes(&repository);
        Document { repository, nodes }
    }

}