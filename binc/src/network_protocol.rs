use std::fmt::{Display, Formatter, Write};
use std::io;
use crate::readwrite::{ReadExt, WriteExt};

pub enum NetworkRequest {
    Disconnect,
    ListFiles{path: String},
    GetFileData{from_revision: u64, path: String},
}

pub enum NetworkResponse {
    ListFiles{files: Vec<String>},
    GetFileData{from_revision: u64, to_revision: u64, data: Vec<u8>},
}

impl NetworkRequest {

    fn message_id(&self) -> u8 {
        match self {
            NetworkRequest::Disconnect => 0,
            NetworkRequest::ListFiles{..} => 1,
            NetworkRequest::GetFileData{..} => 2,
        }
    }

    pub fn read<T: ReadExt>(mut r: &mut T) -> io::Result<NetworkRequest> {
        let message_id = r.read_u8()?;
        match message_id {
            0 => Ok(NetworkRequest::Disconnect),
            1 => {
                let path = r.read_string()?;
                Ok(NetworkRequest::ListFiles{path})
            },
            2 => {
                let from_revision = r.read_length()?;
                let path = r.read_string()?;
                Ok(NetworkRequest::GetFileData{from_revision, path})
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported message id {}", message_id))),
        }
    }

    pub fn write<T: WriteExt>(&self, mut w: &mut T) -> io::Result<()> {
        w.write_u8(self.message_id())?;
        match self {
            NetworkRequest::Disconnect => {},
            NetworkRequest::ListFiles{path} => {
                w.write_string(path)?;
            },
            NetworkRequest::GetFileData{from_revision, path} => {
                w.write_length(*from_revision)?;
                w.write_string(path)?;
            },
        }
        Ok(())
    }
}

impl NetworkResponse {

    fn message_id(&self) -> u8 {
        match self {
            NetworkResponse::ListFiles{..} => 1,
            NetworkResponse::GetFileData{..} => 2,
        }
    }

    pub fn read<T: ReadExt>(mut r: &mut T) -> io::Result<NetworkResponse> {
        let message_id = r.read_u8()?;
        match message_id {
            1 => {
                let files = r.read_string_array()?;
                Ok(NetworkResponse::ListFiles{files})
            },
            2 => {
                let from_revision = r.read_length()?;
                let to_revision = r.read_length()?;
                let data = r.read_bytes()?;
                Ok(NetworkResponse::GetFileData{from_revision, to_revision, data})
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported message id {}", message_id))),
        }
    }

    pub fn write<T: WriteExt>(&self, mut w: &mut T) -> io::Result<()> {
        w.write_u8(self.message_id())?;
        match self {
            NetworkResponse::ListFiles { files } => {
                w.write_string_array(files)?;
            },
            NetworkResponse::GetFileData { from_revision, to_revision, data } => {
                w.write_length(*from_revision)?;
                w.write_length(*to_revision)?;
                w.write_bytes(data)?;
            },
        }
        Ok(())
    }
}

impl Display for NetworkRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkRequest::Disconnect => {
                write!(f, "Disconnect")
            },
            NetworkRequest::ListFiles { path } => {
                write!(f, "ListFiles: {}", path)
            },
            NetworkRequest::GetFileData { from_revision, path } => {
                write!(f, "GetFileData: {} {}..", path, from_revision)
            },
        }
    }
}

impl Display for NetworkResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkResponse::ListFiles { files } => {
                write!(f, "ListFiles: {} files", files.len())
            },
            NetworkResponse::GetFileData { from_revision, to_revision, data } => {
                write!(f, "GetFileData: {}..{}, {} bytes", from_revision, to_revision, data.len())
            },
        }
    }
}