use crate::change::Change;
use crate::changes::Changes;
use crate::node_id::{NodeId, NodeIdGenerator};
use crate::node_store::NodeStore;
use crate::repository::Repository;
use std::io;
use std::io::{Read, Write};

pub struct Document {
    /// Repository containing all revisions
    pub repository: Repository,
    /// This is a cache of the current state of the document, as of the last revision and all pending changes
    pub nodes: NodeStore,
    /// Changes that have been undone and can be redone
    pub undo_changes: Vec<Change>,
    node_id_generator: NodeIdGenerator,
}

fn compute_nodes(repository: &Repository) -> NodeStore {
    let mut nodes: NodeStore = NodeStore::new();
    for change in &repository.changes {
        change.apply(&mut nodes);
    }
    nodes
}

impl Default for Document {
    fn default() -> Self {
        Document {
            repository: Repository::new(),
            nodes: NodeStore::new(),
            undo_changes: Vec::new(),
            node_id_generator: NodeIdGenerator::new(),
        }
    }
}

impl Document {
    pub fn next_id(&mut self) -> NodeId {
        self.node_id_generator.next_id()
    }

    pub fn new(repository: Repository) -> Document {
        let nodes = compute_nodes(&repository);
        Document {
            repository,
            nodes,
            undo_changes: vec![],
            node_id_generator: NodeIdGenerator::new(),
        }
        
    }

    pub fn read<T: Read>(file: &mut T) -> io::Result<Document> {
        let repository = Repository::read(file)?;
        Ok(Self::new(repository))
    }

    fn rebuild(&mut self) {
        self.nodes = compute_nodes(&self.repository);
    }

    pub fn write<T: Write>(&self, w: &mut T) -> io::Result<()> {
        self.repository.write(w)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn find_roots(&self) -> &Vec<NodeId> {
        self.nodes.find_roots()
    }

    pub fn add_and_apply_changes(&mut self, changes: Changes) -> &mut Self
    {
        for change in changes.changes {
            self.add_and_apply_change(change);
        }
        self
    }

    pub fn add_and_apply_change(&mut self, change: Change) {
        self.undo_changes.clear();
        change.apply(&mut self.nodes);
        self.repository.add_change(change);

       /* let last_change = self.pending_changes.changes.last();
        let combined_change = if last_change.is_some() {
            change.combine_change(last_change.unwrap())
        } else {
            None
        };

        if let Some(combined_change) = combined_change {
            self.pending_changes.changes.pop();
            self.pending_changes.changes.push(combined_change);
        } else {
            self.pending_changes.changes.push(change);
        }*/
    }

    pub fn append_and_apply<T: Read>(&mut self, r: &mut T) -> io::Result<()> {
        let from = self.num_change();
        self.repository.append(r)?;
        let to = self.num_change();

        for i in from..to {
            let change = &self.repository.changes[i as usize];
            change.apply(&mut self.nodes);
        }

        Ok(())
    }

    pub fn commit_changes(&mut self) {
        todo!("replace with snapshot")
    }

    pub fn uncommit_changes(&mut self) {
        /*if self.pending_changes.changes.is_empty() && !self.repository.revisions.is_empty() {
            let last_revision = self.repository.revisions.pop().unwrap();
            self.pending_changes = Box::new(last_revision);
            self.rebuild();
        }
        */
        todo!();
    }

    pub fn num_change(&self) -> u64 {
        self.repository.changes.len() as u64
    }

    pub fn undo(&mut self) {
        // TODO use Rewind
    }

    pub fn redo(&mut self) {
        // TODO modify last Rewind
    }
}
