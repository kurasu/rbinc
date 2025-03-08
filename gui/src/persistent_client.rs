use binc::client::Client;
use binc::document::Document;
use binc::network_protocol::{NetworkRequest, NetworkResponse};
use std::io;

pub struct PersistentClient {
    pub client: Client,
    current_pos: u64,
    path: String,
}

impl PersistentClient {
    pub fn connect_to_document(url: &str) -> io::Result<(PersistentClient, Document)> {
        if let Some((host, path)) = url.split_once('/') {
            if let Ok(mut client) = Client::new(host) {
                if let Ok(repo) = client
                    .request(NetworkRequest::GetFileData {
                        from: 0,
                        path: path.to_string(),
                    })?
                    .as_journal()
                {
                    let document = Document::new(repo);
                    Ok((
                        PersistentClient {
                            client,
                            current_pos: document.num_operations() as u64,
                            path: path.to_string(),
                        },
                        document,
                    ))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to get file data",
                    ))
                }
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to connect to host",
                ))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid URL"))
        }
    }

    pub fn check_for_updates(&mut self, document: &mut Document) -> io::Result<()> {
        if let Ok(response) = self.client.request(NetworkRequest::GetFileData {
            path: self.path.clone(),
            from: self.current_pos as u64,
        }) {
            match response {
                NetworkResponse::GetFileData { from, to, data } => {
                    if from != self.current_pos {
                        return Err(io::Error::new(io::ErrorKind::Other, "Revision mismatch"));
                    }
                    if to > from {
                        document.append_and_apply(&mut data.as_slice())?;
                        self.current_pos = to;
                    }

                    Ok(())
                }
                _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid response")),
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get file data",
            ))
        }
    }

    pub fn commit_changes(&mut self, document: &Document) -> io::Result<()> {
        let from = self.current_pos;
        let to = document.journal.operations.len() as u64;

        if to > from {
            let mut data = vec![];
            let mut index = 0;
            for change in &document.journal.operations {
                if index >= from as usize {
                    change.write(&mut data)?;
                }
                index += 1;
            }
            let response = self.client.request(NetworkRequest::AppendFile {
                from,
                to,
                path: self.path.clone(),
                data,
            })?;
            match response {
                NetworkResponse::AppendFile { result } => match result {
                    Ok(()) => {
                        self.current_pos = to;
                    }
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
                },
                _ => return Err(io::Error::new(io::ErrorKind::Other, "Invalid response")),
            }
        }

        Ok(())
    }
}
