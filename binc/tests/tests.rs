#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Cursor, Read, Write};
    use uuid::Uuid;
    use binc::repository::*;
    use binc::revision::*;
    use binc::change::*;
    use binc::document::*;
    use binc::id::Id;

    #[test]
    fn test_create_example_document() {
        let d = create_example_repository();
        assert_eq!(d.revisions.len(), 1);
    }

    fn create_example_repository() -> Repository {
        let mut repo = Repository::new();

        let mut rev = Revision::new();
        let id = Id::default();
        let id2 = Id::default();
        rev.add_change(Change::AddNode{id});
        rev.add_change(Change::RemoveNode{id});
        rev.add_change(Change::AddNode{id: id2});
        rev.add_change(Change::SetString{node: id2, attribute: "name".to_string(), value: "my value".to_string()});
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

    fn read_file(path: &str) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::<u8>::new();
        file.read_to_end(&mut buf).unwrap();
        buf
    }

    #[test]
    fn load_existing_file() {
        let path = "test_data/checklistfile.binc";
        assert!(fs::metadata(path).is_ok());
        let mut file = File::open(path).unwrap();
        let repo = Repository::read(&mut file).unwrap();
        assert!(!repo.revisions.is_empty(), "Repository should have at least one revision");
        let doc = Document::new(repo);
        assert_eq!(doc.node_count(), 8);
        let mut copy: Vec<u8> = vec!();
        doc.repository.write(&mut copy).unwrap();
        let original = read_file(path);

        assert_eq!(copy.len(), original.len(), "File size should be the same");
        assert_eq!(copy, original, "File content should be the same");
    }
}