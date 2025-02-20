mod server;
mod store;

use crate::store::Store;
use binc::attributes::AttributeValue;
use binc::client::Client;
use binc::document::Document;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::node_id::NodeId;
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
    Print { path: String },

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
            Commands::Print { path } => {
                println!("Printing document tree for {}", path);
                if let NetworkResponse::GetFileData {
                    from_revision,
                    to_revision,
                    data,
                } = client.request(NetworkRequest::GetFileData {
                    from_revision: 0,
                    path,
                })? {
                    let repo = Repository::read(&mut &data[..])?;
                    let document = Document::new(repo);

                    let roots = document.find_roots();
                    for root in roots {
                        print_tree(&document, *root, 0);
                    }
                }
            }
            Commands::History { store } => {
                println!("Listing revisions for path {}", store);
                if let NetworkResponse::ListFiles { files } =
                    client.request(NetworkRequest::ListFiles { path: store })?
                {
                    let mut index = 1;
                    for file in files {
                        println!("{}: {}", index, file);
                        index += 1;
                    }
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
            println!("Listing revisions for store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let mut index = 1;
            for rev in &repo.revisions {
                println!(
                    "{}: {} {} {} {}",
                    index, rev.user_name, rev.date, rev.id, rev.message
                );

                for c in &rev.changes {
                    println!("  {}", c);
                }

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
        Commands::Print { path: store } => {
            println!("Printing store {}", store);

            let repo = Repository::read(&mut std::fs::File::open(store)?)?;
            let document = Document::new(repo);

            let roots = document.find_roots();
            for root in roots {
                print_tree(&document, *root, 0);
            }

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

fn get_label(id_string: String, name: Option<&AttributeValue>) -> String {
    if let Some(name) = name {
        let name = name;
        format!("{}", name)
    } else {
        id_string
    }
}

fn print_tree(document: &Document, id: NodeId, depth: i32) {
    if let Some(node) = document.nodes.get(id) {
        let children = &node.children;
        let id_string = format!("ID{}", id);
        let name = node.attributes.get("name");
        let label = get_label(id_string, name);

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

        for child_id in children {
            print_tree(document, child_id.clone(), depth + 1);
        }
    }
}
