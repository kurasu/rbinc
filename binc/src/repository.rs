use io::Write;
use std::io;
use std::io::Read;
use crate::changes::Changes;
use crate::readwrite::{ReadExt, WriteExt};
use crate::revision::*;

pub struct Repository {
    pub revisions: Vec<Revision>
}

impl From<Changes> for Repository {
    fn from(changes: Changes) -> Repository {
        let mut r = Self::new();
        r.add_revision(Revision::from(changes));
        r
    }
}

impl Repository {
    pub const CONTAINER_ID: u32 = 0x42494E43;
    pub const CONTAINER_VERSION: u32 = 1;

    pub fn new() -> Repository {
        Repository { revisions: Vec::new() }
    }

    pub fn add_revision(&mut self, revision: Revision) {
        self.revisions.push(revision);
    }

    pub fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_u32(Repository::CONTAINER_ID)?;
        w.write_u32(Repository::CONTAINER_VERSION)?;

        for revision in &self.revisions {
            revision.write(w)?;
        }
        Ok(())
    }

    pub fn read(mut r: &mut dyn Read) -> io::Result<Repository> {
        let mut doc = Repository::new();
        let container_id = r.read_u32()?;
        let container_version = r.read_u32()?;

        if container_id != Repository::CONTAINER_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        } else if container_version != Repository::CONTAINER_VERSION {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        while let Ok(revision) = Revision::read(r) {
            doc.add_revision(revision);
        }

        Ok(doc)
    }

    pub fn append<T: Read>(&mut self, mut r: &mut T) -> io::Result<()> {
        while let Ok(revision) = Revision::read(&mut r) {
            self.add_revision(revision);
        }
        Ok(())
    }
}