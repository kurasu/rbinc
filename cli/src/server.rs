use crate::store::Store;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use std::io;
use std::net::{TcpListener, TcpStream};

struct Connection {
    stream: TcpStream,
    store: Store,
}

pub(crate) fn server(store: String, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let store = store.clone();
                std::thread::spawn(move || {
                    let peer = s.peer_addr().unwrap();
                    println!("{} connected", peer);

                    let mut connection = Connection::new(s, store);
                    if let Err(e) = connection.handle_connection() {
                        println!("Error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

impl Connection {
    fn new(stream: TcpStream, root_dir: String) -> Connection {
        Connection {
            stream,
            store: Store::new(&root_dir),
        }
    }

    pub fn handle_connection(&mut self) -> io::Result<()> {
        loop {
            let mut stream = &self.stream;
            let peer = stream.peer_addr()?;
            let request = NetworkRequest::read(&mut stream);

            if let Ok(request) = request {
                println!("{peer} request: {}", request);

                match request {
                    NetworkRequest::Disconnect => {
                        return Ok(());
                    }
                    NetworkRequest::ListFiles { path } => {
                        NetworkResponse::ListFiles {
                            files: self.store.list_files(path)?,
                        }
                        .write(&mut stream)?;
                    }
                    NetworkRequest::CreateFile { path } => {
                        NetworkResponse::CreateFile {
                            result: self.store.create_file(path).map_err(|e| e.to_string()),
                        }
                        .write(&mut stream)?;
                    }
                    NetworkRequest::GetFileData {
                        from: from_revision,
                        path,
                    } => {
                        if let Ok((from_revision, to_revision, data)) =
                            self.store.get_file_data(from_revision, path)
                        {
                            NetworkResponse::GetFileData {
                                from: from_revision,
                                to: to_revision,
                                data,
                            }
                            .write(&mut stream)?;
                        }
                    }
                    NetworkRequest::AppendFile {
                        from: from_revision,
                        to: to_revision,
                        path,
                        data,
                    } => {
                        NetworkResponse::AppendFile {
                            result: self
                                .store
                                .append_file(from_revision, to_revision, &path, data)
                                .map_err(|e| e.to_string()),
                        }
                        .write(&mut stream)?;
                    }
                }
            } else if let Err(request) = request {
                return Err(request);
            }
        }
    }
}
