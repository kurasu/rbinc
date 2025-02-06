mod server;

use clap::{Args, Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    CreateStore { filename: String },
    AddNode { store: String, uuid: Uuid },
    RemoveNode { store: String, uuid: Uuid },
    AddChild { store: String, parent: Uuid, child: Uuid },
    History { store: String },
    Serve { store: String, port: u16 },
}

fn main() {

    let matches = Cli::parse();

    println!("{:?}", matches);

    match matches.command {
        Commands::CreateStore { filename } => {
            println!("Creating store {}", filename);
        }
        Commands::AddNode { store, uuid } => {
            println!("Adding node {} to store {}", uuid, store);
        }
        Commands::RemoveNode { store, uuid } => {
            println!("Removing node {} from store {}", uuid, store);
        }
        Commands::AddChild { store, parent, child } => {
            println!("Adding child {} to parent {} in store {}", child, parent, store);
        }
        Commands::History { store } => {
            println!("Listing revisions for store {}", store);
        }
        Commands::Serve { store, port } => {
            println!("Serving store {} on port {}", store, port);
            server::server();
        }
    }
}
