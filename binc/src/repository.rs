use crate::changes::Changes;
use crate::operation::Operation;
use crate::readwrite::{ReadExt, WriteExt};
use io::Write;
use std::io;
use std::io::Read;

pub struct Repository {
    pub operations: Vec<Operation>,
}

impl From<Changes> for Repository {
    fn from(changes: Changes) -> Repository {
        let mut r = Self::new();
        r.operations = changes.operations;
        r
    }
}

impl Repository {
    pub const CONTAINER_ID: u32 = u32::from_be_bytes(*b"binc");
    pub const CONTAINER_VERSION: u32 = 1;

    pub fn new() -> Repository {
        Repository {
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, change: Operation) {
        self.operations.push(change);
    }

    pub fn add_operations(&mut self, changes: Changes) {
        for c in changes.operations {
            self.add_operation(c);
        }
    }

    pub fn write<T: Write>(&self, w: &mut T) -> io::Result<()> {
        w.write_u32(Repository::CONTAINER_ID)?;
        w.write_u32(Repository::CONTAINER_VERSION)?;

        for change in &self.operations {
            change.write(w)?
        }
        Ok(())
    }

    pub fn read<T: Read>(r: &mut T) -> io::Result<Repository> {
        let mut repo = Repository::new();
        let container_id = r.read_u32()?;
        let container_version = r.read_u32()?;

        if container_id != Repository::CONTAINER_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        } else if container_version != Repository::CONTAINER_VERSION {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        while let Ok(change) = Operation::read(r) {
            repo.add_operation(change);
        }

        Ok(repo)
    }

    pub fn append<T: Read>(&mut self, mut r: &mut T) -> io::Result<()> {
        while let Ok(operation) = Operation::read(&mut r) {
            self.add_operation(operation);
        }
        Ok(())
    }
}
