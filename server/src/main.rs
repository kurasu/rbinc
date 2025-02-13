mod server;

use std::io;
use clap::{Parser, Subcommand};
use binc::document::{AttributeValue, Document};
use binc::node_id::NodeId;
use binc::repository::Repository;
use binc::util::shorten_uuid;

/// A simple command line tool for creating, manipulating, viewing and serving BINC documents
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new store
    CreateStore { filename: String },

    /// Print the history of the document
    History { store: String },

    /// Print the document tree
    Print { store: String },

    /// Serve the document over HTTP
    Serve { store: String, port: u16 },
}

fn main() -> io::Result<()> {

    let matches = Cli::parse();

    match matches.command {
        Commands::CreateStore { filename } => {
            println!("Creating store {}", filename);
            Repository::new().write(&mut std::fs::File::create(filename)?)
        }
        Commands::History { store } => {
            println!("Listing revisions for store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let mut index = 1;
            for rev in &repo.revisions {
                println!("{}: {} {} {} {}", index, rev.user_name, rev.date, rev.id, rev.message);
                index += 1;
            }

            Ok(())
        }
        Commands::Print { store } => {
            println!("Printing store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let document = Document::new(repo);

            let roots = document.find_roots();
            for root in roots {
                print_tree(&document, &root, 0);
            }

            Ok(())
        }
        Commands::Serve { store, port } => {
            println!("Serving store {} on port {}", store, port);
            server::server();
            Ok(())
        }
    }
}

fn get_label(id_string: String, name: Option<&AttributeValue>) -> String {
    if let Some(name) = name {
        let name = name;
        format!("{}", name)
    } else { id_string }
}

fn print_tree(document: &Document, root: &NodeId, depth: i32) {
    if let Some(node) = document.nodes.get(root) {
        let children = &node.children.clone();
        let id_string = format!("ID: {:?}", shorten_uuid(root));
        let name = node.attributes.get("name");
        let label = get_label(id_string, name);

        for _ in 0..depth {
            print!("  ");
        }

        print!("{}", label);
        if node.attributes.len() > 0 {
            print!(" (");
            let mut first = true;
            node.attributes.iter().for_each(|(key, value)| {
                if !first {
                    print!(", ");
                }
                print!("{}: {}", key, value);
                first = false;
            });
            print!(")");
        }
        println!();

        for child_id in children {
            print_tree(document, child_id, depth + 1);
        }
    }
}


