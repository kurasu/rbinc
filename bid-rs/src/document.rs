use io::Write;
use std::io;
use crate::iowrappers::WriteExt;
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
}