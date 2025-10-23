use crate::store::Store;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use binc::readwrite::{ReadExt, WriteExt};
use std::io;
use std::net::TcpListener;

struct Connection<T> {
    stream: T,
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

pub(crate) fn ws_server(store: String, port: u16) {
    use std::io::{Read, Write};
    use tungstenite::{accept, Message};

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).unwrap();
    println!("WebSocket server listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let store = store.clone();
                std::thread::spawn(move || {
                    let peer = s.peer_addr().unwrap();
                    println!("{} connected via WebSocket", peer);

                    match accept(s) {
                        Ok(mut websocket) => {
                            let store = Store::new(&store);
                            loop {
                                match websocket.read() {
                                    Ok(Message::Binary(data)) => {
                                        let mut cursor = std::io::Cursor::new(&data);
                                        match NetworkRequest::read(&mut cursor) {
                                            Ok(request) => {
                                                println!("{peer} request: {}", request);

                                                let mut response_buf = Vec::new();
                                                match handle_request(request, &store) {
                                                    Ok(response) => {
                                                        if let Err(e) =
                                                            response.write(&mut response_buf)
                                                        {
                                                            eprintln!(
                                                                "Failed to serialize response: {}",
                                                                e
                                                            );
                                                            break;
                                                        }
                                                    }
                                                    Err(disconnect) => {
                                                        if disconnect {
                                                            println!("{} disconnected", peer);
                                                            break;
                                                        }
                                                    }
                                                }

                                                if let Err(e) =
                                                    websocket.send(Message::Binary(response_buf))
                                                {
                                                    eprintln!("WebSocket send error: {}", e);
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to parse request: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        println!("{} closed connection", peer);
                                        break;
                                    }
                                    Ok(_) => {
                                        // Ignore other message types (text, ping, pong)
                                    }
                                    Err(e) => {
                                        eprintln!("WebSocket error: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("WebSocket handshake error: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

/// Handle a single request and return the response.
/// Returns Err(true) if the connection should be closed (disconnect).
/// Returns Err(false) if there was an error but connection can continue.
fn handle_request(request: NetworkRequest, store: &Store) -> Result<NetworkResponse, bool> {
    match request {
        NetworkRequest::Disconnect => {
            Err(true) // Signal to disconnect
        }
        NetworkRequest::ListFiles { path } => match store.list_files(path) {
            Ok(files) => Ok(NetworkResponse::ListFiles { files }),
            Err(_) => Err(false),
        },
        NetworkRequest::CreateFile { path } => Ok(NetworkResponse::CreateFile {
            result: store.create_file(path).map_err(|e| e.to_string()),
        }),
        NetworkRequest::GetFileData {
            from: from_revision,
            path,
        } => {
            if let Ok((from_revision, to_revision, data)) = store.get_file_data(from_revision, path)
            {
                Ok(NetworkResponse::GetFileData {
                    from: from_revision,
                    to: to_revision,
                    data,
                })
            } else {
                Err(false)
            }
        }
        NetworkRequest::AppendFile {
            from: from_revision,
            to: to_revision,
            path,
            data,
        } => Ok(NetworkResponse::AppendFile {
            result: store
                .append_file(from_revision, to_revision, &path, data)
                .map_err(|e| e.to_string()),
        }),
    }
}

impl<T: ReadExt + WriteExt> Connection<T> {
    fn new(stream: T, root_dir: String) -> Connection<T> {
        Connection {
            stream,
            store: Store::new(&root_dir),
        }
    }

    pub fn handle_connection(&mut self) -> io::Result<()> {
        loop {
            let request = NetworkRequest::read(&mut self.stream)?;

            println!("request: {}", request);

            match handle_request(request, &self.store) {
                Ok(response) => {
                    response.write(&mut self.stream)?;
                }
                Err(disconnect) => {
                    if disconnect {
                        return Ok(());
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Request handling error",
                        ));
                    }
                }
            }
        }
    }
}
