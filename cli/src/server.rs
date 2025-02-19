use std::{fs, io};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::readwrite::*;

struct Connection {
    stream: TcpStream,
    store: String,
}

pub(crate) fn server(store: String, port: u16) {
    let addr = format!("localhost:{}", port);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let s = stream.unwrap();

        let peer = s.peer_addr().unwrap();
        println!("Connection established from {}", peer);

        let mut connection = Connection::new(s, store.clone());
        let r = connection.handle_connection();
        if let Err(r) = r { println!("Error: {}", r) }

        println!("Connection sed from {}", peer);
    }
}

impl Connection {

    fn new(stream: TcpStream, store: String) -> Connection {
        Connection { stream, store }
    }

    pub fn handle_connection(&mut self) -> io::Result<()>{
        loop {
            let mut stream = &self.stream;
            let request = NetworkRequest::read(&mut stream);

            if let Ok(request) = request
            {
                println!("Request: {}", request);

                match request {
                    NetworkRequest::Disconnect => {
                        println!("Closing connection");
                        return Ok(());
                    },
                    NetworkRequest::ListFiles{ path } => {
                        NetworkResponse::ListFiles { files: self.list_files(path)? }.write(&mut stream)?;
                    },
                    NetworkRequest::GetFileData { from_revision, path } => {
                        if let Ok((from_revision, to_revision , data)) = self.get_file_data(from_revision, path) {
                            NetworkResponse::GetFileData { from_revision, to_revision , data}.write(&mut stream)?;
                        }
                    },

                }
            }
            else if let Err(request) = request {
                return Err(request);
            }
        }
    }

    fn list_files(&self, path: String) -> io::Result<Vec<String>>{
        let dir = self.store.clone() + "/" + &path;
        let entries = fs::read_dir(dir)?;

        let filenames: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| e.file_name().into_string().ok())
            })
            .collect();

        Ok(filenames)
    }

    fn get_file_data(&self, _from_revision: u64, _path: String) -> io::Result<(u64, u64, Vec<u8>)> {
        Ok((0, 0, vec![]))
    }
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