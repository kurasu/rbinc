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
                    Ok((PersistentClient { client, current_revision: 0, path: path.to_string() }, document))
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
            from_revision: self.current_revision
        }) {
            match response {
                NetworkResponse::GetFileData { from_revision, to_revision, data } => {
                    if from_revision != self.current_revision {
                        return Err(io::Error::new(io::ErrorKind::Other, "Revision mismatch"));
                    }
                    if to_revision > from_revision {
                        document.repository.append(&mut data.as_slice())?;
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
}