use io::Write;
use std::io;
use uuid::Uuid;
use crate::iowrappers::WriteExt;
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

    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_u32(Document::CONTAINER_ID)?;

        for revision in &self.revisions {
            revision.write(w)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

    #[test]
    fn test_create_example_document() {
        let mut d = create_example_document();
        assert_eq!(d.revisions.len(), 1);
    }

    fn create_example_document() -> Document {
        let mut d = Document::new();

        let mut r = Revision::new();
        let uuid = Uuid::new_v4();
        r.add_change(Box::new(AddNode::new(uuid)));
        d.add_revision(r);
        d
    }

    #[test]
    fn save_example_document() {
        let mut d = create_example_document();
        let mut file = File::create("output.abc").unwrap();
        d.write(&mut file).unwrap();
        file.flush().unwrap();
    }
}