use io::Write;
use std::io;
use std::io::Read;
use crate::change::Change;
use crate::changes::Changes;
use crate::readwrite::{ReadExt, WriteExt};

pub struct Repository {
    pub changes: Vec<Change>,
}

impl From<Changes> for Repository {
    fn from(changes: Changes) -> Repository {
        let mut r = Self::new();
        r.changes = changes.changes;
        r
    }
}

impl Repository {
    pub const CONTAINER_ID: u32 = u32::from_le_bytes(*b"binc");
    pub const CONTAINER_VERSION: u32 = 1;

    pub fn new() -> Repository {
        Repository { changes: Vec::new() }
    }

    pub fn add_change(&mut self, change: Change) {
        self.changes.push(change);
    }

    pub fn add_changes(&mut self, changes: Changes) {
        for c in changes.changes {
            self.add_change(c);
        }
    }

    pub fn write<T: Write>(&self, mut w: &mut T) -> io::Result<()> {
        w.write_u32(Repository::CONTAINER_ID)?;
        w.write_u32(Repository::CONTAINER_VERSION)?;

        for change in &self.changes {
            self.write_change(w, change)?
        }
        Ok(())
    }

    fn write_change<T: Write>(&self, w: &mut T, change: &Change) -> io::Result<()> {
        w.write_length(change.change_type())?;
        let mut temp: Vec<u8> = vec![];
        change.write(&mut temp)?;
        w.write_length(temp.len() as u64)?;
        w.write_all(&temp)
    }

    pub fn read<T: Read>(mut r: &mut T) -> io::Result<Repository> {
        let mut repo = Repository::new();
        let container_id = r.read_u32()?;
        let container_version = r.read_u32()?;

        if container_id != Repository::CONTAINER_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        } else if container_version != Repository::CONTAINER_VERSION {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        while let Ok(change) = Change::read(r) {
            repo.add_change(change);
        }

        Ok(repo)
    }

    pub fn append<T: Read>(&mut self, mut r: &mut T) -> io::Result<()> {
        while let Ok(change) = Change::read(&mut r) {
            self.add_change(change);
        }
        Ok(())
    }
}