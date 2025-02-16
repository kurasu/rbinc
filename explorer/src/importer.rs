use crate::NodeId;
use std::io;
use std::io::{BufReader, Read};
use xml::EventReader;
use xml::reader::XmlEvent;
use binc::changes::Changes;
use binc::node_id::NodeIdGenerator;
use binc::repository::Repository;

pub const IMPORTERS: [Importer; 1] = [Importer::XML];

pub enum Importer {
    XML,
}

pub trait Import {
    fn import<R: Read>(&self, reader: &mut R) -> io::Result<Repository>;
    fn get_name(&self) -> &str;
    fn file_extensions(&self) -> Vec<&str>;
}

impl Import for Importer {
    fn import<R: Read>(&self, reader: &mut R) -> io::Result<Repository> {
        match self {
            Importer::XML => {
                import_xml(reader)
            }
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Importer::XML => "XML",
        }
    }

    fn file_extensions(&self) -> Vec<&str> {
        match self {
            Importer::XML => vec!["xml"],
        }
    }
}

fn import_xml<R: Read>(reader: &mut R) -> io::Result<Repository> {
    let parser = EventReader::new(reader);
    let mut changes = Changes::new();
    let mut depth = 0;
    let mut parent_id_stack = Vec::<NodeId>::new();
    parent_id_stack.push(NodeId::ROOT_NODE);
    let mut id_provider = NodeIdGenerator::new();

    let mut current_id = NodeId::NO_NODE;

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, namespace }) => {
                //println!("{:spaces$}+{name}", "", spaces = depth * 2);
                current_id = id_provider.next_id();
                let parent_id = parent_id_stack.last().expect("StartElement/EndElement mismatch");
                changes.add_node(current_id, *parent_id, 0);
                changes.set_string(current_id, "xml:name", name.local_name.as_str());

                for attr in attributes {
                    let key = format!("attr:{}", attr.name.local_name);
                    changes.set_string(current_id, key.as_str(), attr.value.as_str());
                }
                depth += 1;
                parent_id_stack.push(current_id);
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                //println!("{:spaces$}-{name}", "", spaces = depth * 2);
                parent_id_stack.pop();
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }

    Ok(Repository::from(changes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use binc::document::Document;

    #[test]
    fn test_import_xml() {
        let xml_data = r#"
            <root>
                <child attr="value">Text</child>
            </root>
        "#;
        let cursor = Cursor::new(xml_data);
        let mut reader = BufReader::new(cursor);

        let result = import_xml(&mut reader);
        assert!(result.is_ok());

        let document = Document::new(result.unwrap());
        // Add more assertions to verify the contents of the repository
    }
}
