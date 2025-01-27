#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Cursor, Read, Write};
    use uuid::Uuid;
    use bid_rs::document::*;
    use bid_rs::revision::*;

    #[test]
    fn test_create_example_document() {
        let d = create_example_document();
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
        let d = create_example_document();
        //let mut file = File::create("target/exa.abc").unwrap();
        //d.write(&mut file).unwrap();
        //file.flush().unwrap();
        let mut buf = Vec::<u8>::new();
        d.write(&mut buf).unwrap();
    }

    #[test]
    fn save_and_load_example_document() {
        let d = create_example_document();
        let mut buf = Vec::<u8>::new();
        d.write(&mut buf).unwrap();
        let mut r = Cursor::new(buf);
        let d2 = Document::read(&mut r).unwrap();
        assert_eq!(d.revisions.len(), d2.revisions.len());
    }
}