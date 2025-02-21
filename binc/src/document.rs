use crate::change::Change;
use crate::changes::Changes;
use crate::node_id::{NodeId, NodeIdGenerator};
use crate::node_store::NodeStore;
use crate::repository::Repository;
use crate::revision::Revision;
use std::io;
use std::io::{Read, Write};

pub struct Document {
    /// Repository containing all revisions
    pub repository: Repository,
    /// This is a cache of the current state of the document, as of the last revision and all pending changes
    pub nodes: NodeStore,
    /// Changes that have not been committed to the repository
    pub pending_changes: Box<Revision>,
    /// Changes that have been undone and can be redone
    pub undo_changes: Vec<Change>,
    node_id_generator: NodeIdGenerator,
}

fn compute_nodes(repository: &Repository) -> NodeStore {
    let mut nodes: NodeStore = NodeStore::new();
    for rev in &repository.revisions {
        for change in &rev.changes {
            change.apply(&mut nodes);
        }
    }
    nodes
}

impl Default for Document {
    fn default() -> Self {
        Document {
            repository: Repository::new(),
            nodes: NodeStore::new(),
            pending_changes: Box::new(Revision::new(None)),
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
        let pending_changes = Box::new(Revision::new(repository.revisions.last()));
        Document {
            repository,
            nodes,
            pending_changes,
            undo_changes: vec![],
            node_id_generator: NodeIdGenerator::new(),
        }
        
    }

    pub fn read(file: &mut dyn Read) -> io::Result<Document> {
        let repository = Repository::read(file)?;
        Ok(Self::new(repository))
    }

    fn rebuild(&mut self) {
        self.nodes = compute_nodes(&self.repository);

        for change in &self.pending_changes.changes {
            change.apply(&mut self.nodes);
        }
    }

    pub fn write(&self, w: &mut dyn Write) -> io::Result<()> {
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

        let last_change = self.pending_changes.changes.last();
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
        }
    }

    pub fn append_and_apply<T: Read>(&mut self, mut r: &mut T) -> io::Result<()> {
        let from = self.num_revisions();
        self.repository.append(r)?;
        let to = self.num_revisions();

        for i in from..to {
            let revision = &self.repository.revisions[i as usize];
            for change in &revision.changes {
                change.apply(&mut self.nodes);
            }
        }

        Ok(())
    }

    pub fn commit_changes(&mut self) {
        let src = Box::new(Revision::new(Some(&self.pending_changes)));
        let pending = std::mem::replace(&mut self.pending_changes, src);
        self.repository.add_revision(*pending);
    }

    pub fn uncommit_changes(&mut self) {
        if self.pending_changes.changes.is_empty() && !self.repository.revisions.is_empty() {
            let last_revision = self.repository.revisions.pop().unwrap();
            self.pending_changes = Box::new(last_revision);
            self.rebuild();
        }
    }

    pub fn num_revisions(&self) -> u64 {
        self.repository.revisions.len() as u64
    }

    pub fn undo(&mut self) {
        if self.pending_changes.changes.is_empty() {
            self.uncommit_changes();
        }

        if let Some(change) = self.pending_changes.changes.pop() {
            self.undo_changes.push(change);
            self.rebuild();
        }

        self.rebuild();
    }

    pub fn redo(&mut self) {
        if let Some(change) = self.undo_changes.pop() {
            self.pending_changes.changes.push(change);
            self.rebuild();
        }
    }
}
