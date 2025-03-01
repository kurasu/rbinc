#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Cursor, Read, Write};

    use binc::repository::*;

    use binc::changes::Changes;
    use binc::document::*;
    use binc::node_id::{NodeId, NodeIdGenerator};

    #[test]
    fn test_create_example_document() {
        let d = create_example_repository();
        assert_eq!(d.changes.len(), 5);
    }

    fn create_example_repository() -> Repository {
        let mut repo = Repository::new();

        let mut generator = NodeIdGenerator::new();
        let id = generator.next_id();
        let id2 = generator.next_id();

        let mut changes = Changes::new();
        changes.add_node(id, NodeId::ROOT_NODE, 0);
        changes.remove_node(id);
        changes.add_node(id2, NodeId::ROOT_NODE, 0);
        changes.set_string_s(id2, "name", "my value");

        repo.add_changes(changes);
        repo
    }

    #[test]
    fn save_example_document() {
        let repo = create_example_repository();
        let mut buf = Vec::<u8>::new();
        repo.write(&mut buf).unwrap();
        let doc = Document::new(repo);
        assert_eq!(doc.find_roots().len(), 1)
    }

    #[test]
    fn save_and_load_example_document() {
        let repo = create_example_repository();
        let mut buf = Vec::<u8>::new();
        repo.write(&mut buf).unwrap();
        let mut r = Cursor::new(buf);
        let repo2 = Repository::read(&mut r).unwrap();
        assert_eq!(repo.changes.len(), repo2.changes.len());
        let doc = Document::new(repo2);
        assert_eq!(doc.find_roots().len(), 1)
    }

    fn read_file(path: &str) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::<u8>::new();
        file.read_to_end(&mut buf).unwrap();
        buf
    }

    //#[test]
    fn load_existing_file() {
        let path = "test_data/checklistfile.binc";
        assert!(fs::metadata(path).is_ok());
        let mut file = File::open(path).unwrap();
        let repo = Repository::read(&mut file).unwrap();
        assert!(
            !repo.changes.is_empty(),
            "Repository should have at least one change"
        );
        let doc = Document::new(repo);
        assert_eq!(doc.node_count(), 8);
        let mut copy: Vec<u8> = vec![];
        doc.repository.write(&mut copy).unwrap();
        let original = read_file(path);

        assert_eq!(copy.len(), original.len(), "File size should be the same");
        assert_eq!(copy, original, "File content should be the same");
    }
}
