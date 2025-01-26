use io::Write;
use std::io;
use crate::io::WriteExt;
use crate::revision::*;

struct Document {
    revisions: Vec<Revision>
}

impl Document {
    pub const CONTAINER_ID: u32 =  0x484f484e;

    fn new() -> Document {
        Document { revisions: Vec::new() }
    }

    fn add_revision(&mut self, revision: Revision) {
        self.revisions.push(revision);
    }

    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_uint32(Document::CONTAINER_ID)?;

        for revision in &self.revisions {
            revision.write(w)?;
        }
        Ok(())
    }
}