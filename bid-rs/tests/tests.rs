#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Cursor, Read, Write};
    use uuid::Uuid;
    use bid_rs::repository::*;
    use bid_rs::revision::*;
    use bid_rs::changes::*;
    use bid_rs::document::*;

    #[test]
    fn test_create_example_document() {
        let d = create_example_repository();
        assert_eq!(d.revisions.len(), 1);
    }

    fn create_example_repository() -> Repository {
        let mut repo = Repository::new();

        let mut rev = Revision::new();
        let uuid = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        rev.add_change(Box::new(AddNode::new(uuid)));
        rev.add_change(Box::new(RemoveNode::new(uuid)));
        rev.add_change(Box::new(AddNode::new(uuid2)));
        rev.add_change(Box::new(SetString::new(uuid2, "name", "my value".to_string())));
        repo.add_revision(rev);
        repo
    }

    #[test]
    fn save_example_document() {
        let repo = create_example_repository();
        //let mut file = File::create("target/exa.abc").unwrap();
        //d.write(&mut file).unwrap();
        //file.flush().unwrap();
        let mut buf = Vec::<u8>::new();
        repo.write(&mut buf).unwrap();
        let doc = Document::new(repo);
        assert_eq!(doc.node_count(), 1)
    }

    #[test]
    fn save_and_load_example_document() {
        let repo = create_example_repository();
        let mut buf = Vec::<u8>::new();
        repo.write(&mut buf).unwrap();
        let mut r = Cursor::new(buf);
        let repo2 = Repository::read(&mut r).unwrap();
        assert_eq!(repo.revisions.len(), repo2.revisions.len());
        let doc = Document::new(repo2);
        assert_eq!(doc.node_count(), 1)
    }
}