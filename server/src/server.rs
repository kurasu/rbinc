use std::{fs, io};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use bid_rs::iowrappers::*;

pub(crate) fn server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let s = stream.unwrap();

        println!("Connection established from {}", s.peer_addr().unwrap());

        let r = handle_connection(&s);
        if r.is_err() { println!("Error: {}", r.unwrap_err().to_string()) }
    }
}

fn handle_connection(mut stream: &TcpStream) -> io::Result<()>{

    loop {
        let request = stream.read_length();

        if request.is_ok()
        {
            let r = request.unwrap();
            println!("Request: {}", r);

            if r == 0 {
                println!("Closing connection");
                return Ok(());
            }

            let result = match r {
                1=>list_files(&stream),
                2=>create_file(&stream),
                3=>get_file(&stream),
                4=>get_file_changes(&stream),
                5=>send_file_changes(&stream),
                _=>Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported request {}", r))),
            };

            if result.is_err() { return result; }
        }
    }
}

fn list_files(mut stream: &TcpStream) -> io::Result<()>{
    let entries = fs::read_dir("./").unwrap();

    let filenames: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| e.file_name().into_string().ok())
        })
        .collect();

    let mut writer = stream;

    writer.write_length(filenames.len() as u64).unwrap();

    for filename in filenames {
        println!("Name: {}", filename);
        writer.write_string(&filename)?;
    }
    Ok(())
}

fn create_file(mut stream: &TcpStream) -> io::Result<()> {
    let mut reader = &mut stream;
    let filename = reader.read_string()?;
    println!("Create File: {}", filename);
    create_file_with_name(filename)?;
    Ok(())
}

fn create_file_with_name(filename: String) -> io::Result<()> {
    let mut file = File::create(filename)?;
    let header: [u8; 4] = [0x48, 0x48, 0x48, 0x48];
    let format: [u8; 16] = [0; 16];
    let flags: [u8; 8] = [0; 8];
    file.write(&header)?;
    file.write(&format)?;
    file.write(&flags)?;
    Ok(())
}

fn get_file(mut stream: &TcpStream) -> io::Result<()> {
    todo!()
}

fn get_file_changes(mut stream: &TcpStream) -> io::Result<()> {
    todo!()
}

fn send_file_changes(mut stream: &TcpStream) -> io::Result<()> {
    todo!()
}
