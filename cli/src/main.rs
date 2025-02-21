mod server;
mod store;

use crate::store::Store;
use binc::attributes::AttributeValue;
use binc::client::Client;
use binc::document::Document;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::node_id::NodeId;
use binc::node_store::Node;
use binc::repository::Repository;
use clap::{Parser, Subcommand};
use std::io;

/// A simple command line tool for creating, manipulating, viewing and serving BINC documents
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Remote server to connect to
    #[arg(short, long)]
    remote: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the contents of the path
    List { path: String },

    /// Create a new file
    CreateFile { path: String },

    /// Print the history of the document
    History { store: String },

    /// Print the document tree
    Tree { path: String },

    /// Serve the contents of the directory over HTTP
    Serve { path: String, port: u16 },
}

fn main() -> io::Result<()> {
    let matches = Cli::parse();

    if let Some(remote) = matches.remote {
        println!("Connecting to remote server {}", remote);

        let mut client = Client::new(&remote)?;

        match matches.command {
            Commands::List { path } => {
                if let NetworkResponse::ListFiles { files } =
                    client.request(NetworkRequest::ListFiles { path })?
                {
                    list_files(files);
                }
            }
            Commands::CreateFile { path } => {
                if let NetworkResponse::CreateFile { result } =
                    client.request(NetworkRequest::CreateFile { path })?
                {
                    match result {
                        Ok(()) => println!("File created"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            Commands::Tree { path } => {
                println!("Printing document tree for {}", path);
                if let Ok(repo) = client.request(NetworkRequest::GetFileData { from: 0, path })?.into_repository() {
                    let document = Document::new(repo);
                    print_tree(&document, NodeId::ROOT_NODE, 0, 0);
                }
            },
            Commands::History { store: path } => {
                println!("Listing revisions for {}", path);
                if let Ok(repo) = client.request(NetworkRequest::GetFileData { from: 0, path })?.into_repository() {
                    repo.changes.iter().for_each(|c| {
                        println!(" * {}", c);
                    });
                }
            }
            _ => {
                println!("Command not supported for remote server");
            }
        }
        return Ok(());
    }

    let store = Store::new(".");

    match matches.command {
        Commands::CreateFile { path: filename } => {
            println!("Creating file {}", filename);
            store.create_file(filename)
        }
        Commands::History { store } => {
            println!("Listing changes for store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let mut index = 1;

            for c in &repo.changes {
                println!("{}: {}", index, c);
                index += 1;
            }


            Ok(())
        }
        Commands::List { path } => {
            println!("Listing directory {}", path);
            let mut files = vec![];
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                files.push(path.to_string_lossy().to_string());
            }

            list_files(files);

            Ok(())
        }
        Commands::Tree { path: store } => {
            println!("Printing store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let document = Document::new(repo);

            print_tree(&document, NodeId::ROOT_NODE, 0, 0);

            Ok(())
        }
        Commands::Serve { path: store, port } => {
            println!("Serving store {} on port {}", store, port);
            server::server(store, port);
            Ok(())
        }
    }
}

fn list_files(files: Vec<String>) {
    for file in files {
        println!("{}", file);
    }
}

fn get_label(node: &Node, index_in_parent: usize) -> String {
    let name = node.get_name();
    let type_name = node.get_type();

    if let Some(name) = name {
        if let Some(t) = type_name {
            return format!("{}: [{}] {}", index_in_parent, t, name);
        }
        else
        {
            return format!("{}: {}", index_in_parent, name);
        }
    }

    if let Some(t) = type_name {
        return format!("{}: [{}]", index_in_parent, t);
    }

    format!("{}: ID{}", index_in_parent, node.id.index())
}

fn print_tree(document: &Document, id: NodeId, depth: i32, index_in_parent: usize) {
    if let Some(node) = document.nodes.get(id) {
        let children = &node.children;
        let label = get_label(node, index_in_parent);

        for _ in 0..depth {
            print!("  ");
        }

        print!("{}", label);
        if node.attributes.len() > 0 {
            print!(" (");
            let mut first = true;
            node.attributes.iter().for_each(|a| {
                if !first {
                    print!(", ");
                }
                print!("{}: {}", a.key, a.value);
                first = false;
            });
            print!(")");
        }
        println!();

        let mut index = 0;
        for child_id in children {
            print_tree(document, child_id.clone(), depth + 1, index);
            index += 1;
        }
    }
}
