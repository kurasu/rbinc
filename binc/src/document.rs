use crate::changes::Changes;
use crate::journal::Journal;
use crate::node_id::{NodeId, NodeIdGenerator};
use crate::node_store::NodeStore;
use crate::operation::Operation;
use std::io;
use std::io::{Read, Write};

pub struct Document {
    /// Journal containing all revisions
    pub journal: Journal,
    /// This is a cache of the current state of the document, as of the last revision and all pending operations
    pub nodes: NodeStore,
    /// Revision that have been undone to
    pub undo_revision: Option<usize>,
    pub node_id_generator: NodeIdGenerator,
}

fn compute_nodes(journal: &Journal, end_revision: Option<usize>) -> NodeStore {
    let mut nodes: NodeStore = NodeStore::new();

    let to = end_revision.unwrap_or(journal.operations.len());
    for operation in &journal.operations.as_slice()[..to] {
        operation.apply(&mut nodes);
    }
    nodes
}

impl Default for Document {
    fn default() -> Self {
        Document {
            journal: Journal::new(),
            nodes: NodeStore::new(),
            undo_revision: None,
            node_id_generator: NodeIdGenerator::new(),
        }
    }
}

impl Document {
    pub fn next_id(&mut self) -> NodeId {
        self.node_id_generator.next_id()
    }

    pub fn new(journal: Journal) -> Document {
        let nodes = compute_nodes(&journal, None);
        Document {
            journal,
            nodes,
            undo_revision: None,
            node_id_generator: NodeIdGenerator::new(),
        }
    }

    pub fn read<T: Read>(file: &mut T) -> io::Result<Document> {
        let journal = Journal::read(file)?;
        Ok(Self::new(journal))
    }

    fn rebuild(&mut self, end_revision: Option<usize>) {
        self.nodes = compute_nodes(&self.journal, end_revision);
    }

    pub fn write<T: Write>(&self, w: &mut T) -> io::Result<()> {
        self.journal.write(w)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn find_roots(&self) -> &Vec<NodeId> {
        self.nodes.find_roots()
    }

    pub fn add_and_apply_changes(&mut self, changes: Changes) -> &mut Self {
        for change in changes.operations {
            self.add_and_apply(change);
        }
        self
    }

    pub fn add_and_apply(&mut self, operation: Operation) {
        if self.undo_revision.is_some() {
            self.journal
                .operations
                .truncate(self.undo_revision.unwrap() as usize);
            self.undo_revision = None;
        }
        operation.apply(&mut self.nodes);
        self.journal.add_operation(operation);

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
        let from = self.num_operations();
        self.journal.append(r)?;
        let to = self.num_operations();

        for i in from..to {
            let change = &self.journal.operations[i as usize];
            change.apply(&mut self.nodes);
        }

        Ok(())
    }

    pub fn num_operations(&self) -> usize {
        self.journal.operations.len()
    }

    pub fn can_undo(&self) -> bool {
        self.num_operations() > 0
    }

    pub fn can_redo(&self) -> bool {
        self.undo_revision.is_some()
    }

    pub fn undo(&mut self) {
        self.undo_revision = match self.undo_revision {
            Some(rev) => {
                if rev == 0 {
                    return;
                }
                Some(rev - 1)
            }
            None => {
                if self.num_operations() == 0 {
                    return;
                }
                Some(self.num_operations() - 1)
            }
        };

        self.rebuild(self.undo_revision);
    }

    pub fn redo(&mut self) {
        self.undo_revision = match self.undo_revision {
            Some(rev) => {
                if rev + 1 >= self.num_operations() {
                    None
                } else {
                    Some(rev + 1)
                }
            }
            None => None,
        };

        self.rebuild(self.undo_revision);
    }

    pub fn get_or_define_attribute_id(&mut self, key: &str) -> usize {
        match self.nodes.attribute_names.get_index(key) {
            Some(index) => index,
            None => {
                let next_id = self.nodes.attribute_names.len();
                self.add_and_apply(Operation::DefineAttributeName {
                    id: next_id,
                    name: key.to_string(),
                });
                next_id
            }
        }
    }

    pub fn type_name(&self, id: Option<usize>) -> String {
        if let Some(id) = id {
            match self.nodes.type_names.get(id) {
                Some(name) => name.to_string(),
                None => format!("Type #{}", id),
            }
        } else {
            "None".to_string()
        }
    }

    pub fn attribute_name(&self, id: usize) -> String {
        match self.nodes.attribute_names.get(id) {
            Some(name) => name.to_string(),
            None => format!("Attribute #{}", id),
        }
    }

    pub fn tag_name(&self, id: usize) -> String {
        match self.nodes.tag_names.get(id) {
            Some(name) => name.to_string(),
            None => format!("Tag #{}", id),
        }
    }
}
