use io::Write;
use std::io;
use std::io::Read;
use crate::iowrappers::{ReadExt, WriteExt};
use crate::revision::*;

pub struct Document {
    pub revisions: Vec<Revision>
}

impl Document {
    pub const CONTAINER_ID: u32 =  0x484f484e;

    pub fn new() -> Document {
        Document { revisions: Vec::new() }
    }

    pub fn add_revision(&mut self, revision: Revision) {
        self.revisions.push(revision);
    }

    pub fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_u32(Document::CONTAINER_ID)?;

        for revision in &self.revisions {
            revision.write(w)?;
        }
        Ok(())
    }


    pub fn read(mut r: &mut dyn Read) -> io::Result<Document> {
        let mut doc = Document::new();
        let container_id = r.read_u32()?;

        if container_id != Document::CONTAINER_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let revision = Revision::read(r)?;
        doc.add_revision(revision);

        Ok(doc)
    }
}