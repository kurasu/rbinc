use std::{fs, io};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::readwrite::*;

struct Connection {
    stream: TcpStream,
}

pub(crate) fn server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let s = stream.unwrap();

        println!("Connection established from {}", s.peer_addr().unwrap());

        let mut connection = Connection::new(s);
        let r = connection.handle_connection();
        if r.is_err() { println!("Error: {}", r.unwrap_err().to_string()) }
    }
}

impl Connection {

    fn new(stream: TcpStream) -> Connection {
        Connection { stream }
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
        }
    }

    fn list_files(&self, path: String) -> io::Result<Vec<String>>{
        let entries = fs::read_dir(path).unwrap();

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