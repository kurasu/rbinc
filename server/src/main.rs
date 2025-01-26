use std::net::TcpListener;

mod revision;
mod io;

use clap::{arg, Command};

fn cli() -> Command {
    Command::new("xyz")
        .about("XYZ Toy Server")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("create")
                .about("Create new store")
                .arg(arg!(filename: [FILENAME])))
        .subcommand(
            Command::new("add-node")
                .about("Add node")
                .arg(arg!(store: [STORE]))
                .arg(arg!(uuid: [UUID])))
        .subcommand(
            Command::new("remove-node")
                .about("Remove node")
                .arg(arg!(store: [STORE]))
                .arg(arg!(uuid: [UUID])))
        .subcommand(
            Command::new("add-child")
                .about("Add node as child")

                .arg(arg!(store: [STORE]))
                .arg(arg!(parent: [PARENT]))
                .arg(arg!(child: [CHILD])))
        .subcommand(
            Command::new("history")
                .about("List revisions"))
        .subcommand(
            Command::new("serve")
                .about("Start server"))
}

fn main() {
    server()
    //let matches = cli().get_matches();

    /*let c = add(10, 4);

    let uuid1 = uuid::Uuid::new_v4();
    let a = Change::AddNode(uuid1);
    let b = Change::SetBool { node: uuid1, field: "value".to_string(), value: true};
    println!("Hello {}", revision::get_change_id(a));
     */
}
