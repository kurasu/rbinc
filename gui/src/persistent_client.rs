use std::io;
use binc::client::Client;
use binc::document::Document;
use binc::network_protocol::{NetworkRequest, NetworkResponse};

pub struct PersistentClient {
    pub client: Client,
    current_revision: u64,
    path: String,
}

impl PersistentClient {
    pub fn connect_to_document(url: &str) -> io::Result<(PersistentClient, Document)> {
        if let Some((host, path)) = url.split_once('/') {
            if let Ok(mut client) = Client::new(host) {
                if let Ok(repo) = client
                    .request(NetworkRequest::GetFileData {
                        from_revision: 0,
                        path: path.to_string(),
                    })?
                    .into_repository()
                {
                    let document = Document::new(repo);
                    Ok((PersistentClient { client, current_revision: document.num_revisions(), path: path.to_string() }, document))
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
            from_revision: self.current_revision as u64,
        }) {
            match response {
                NetworkResponse::GetFileData { from_revision, to_revision, data } => {
                    if from_revision != self.current_revision {
                        return Err(io::Error::new(io::ErrorKind::Other, "Revision mismatch"));
                    }
                    if to_revision > from_revision {
                        document.append_and_apply(&mut data.as_slice())?;
                        self.current_revision = to_revision;
                    }

                    Ok(())
                },
                _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid response"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to get file data"))
        }
    }

    pub fn commit_changes(&mut self, document: &Document) -> io::Result<()> {
        let from_revision = self.current_revision;
        let to_revision = document.repository.revisions.len() as u64;

        if to_revision > from_revision {
            let mut data = vec![];
            let mut index = 0;
            for r in &document.repository.revisions {
                if index >= from_revision as usize {
                    r.write(&mut data)?;
                }
                index += 1;
            }
            let response = self.client.request(NetworkRequest::AppendFile { from_revision, to_revision, path: self.path.clone(), data })?;
            match response {
                NetworkResponse::AppendFile { result } => {
                    match result {
                        Ok(()) => {
                            self.current_revision = to_revision;
                        },
                        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e))
                    }
                },
                _ => return Err(io::Error::new(io::ErrorKind::Other, "Invalid response"))
            }
        }

        Ok(())
    }
}