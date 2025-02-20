use std::fmt::{Display, Formatter, Write};
use std::io;
use crate::readwrite::{ReadExt, WriteExt};

const DISCONNECT: u8 = 0;
const LIST_FILES: u8 = 1;
const GET_FILE_DATA: u8 = 2;
const CREATE_FILE: u8 = 3;

pub enum NetworkRequest {
    Disconnect,
    ListFiles{path: String},
    CreateFile{path: String},
    GetFileData{from_revision: u64, path: String},
}

pub enum NetworkResponse {
    ListFiles{files: Vec<String>},
    CreateFile{result: Result<(), String>},
    GetFileData{from_revision: u64, to_revision: u64, data: Vec<u8>},
}

impl NetworkRequest {

    fn message_id(&self) -> u8 {
        match self {
            NetworkRequest::Disconnect => DISCONNECT,
            NetworkRequest::ListFiles{..} => LIST_FILES,
            NetworkRequest::GetFileData{..} => GET_FILE_DATA,
            NetworkRequest::CreateFile{..} => CREATE_FILE,
        }
    }

    pub fn read<T: ReadExt>(r: &mut T) -> io::Result<NetworkRequest> {
        let message_id = r.read_u8()?;
        match message_id {
            DISCONNECT => Ok(NetworkRequest::Disconnect),
            LIST_FILES => {
                let path = r.read_string()?;
                Ok(NetworkRequest::ListFiles{path})
            },
            GET_FILE_DATA => {
                let from_revision = r.read_length()?;
                let path = r.read_string()?;
                Ok(NetworkRequest::GetFileData{from_revision, path})
            },
            CREATE_FILE => {
                let path = r.read_string()?;
                Ok(NetworkRequest::CreateFile{path})
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported message id {}", message_id))),
        }
    }

    pub fn write<T: WriteExt>(&self, w: &mut T) -> io::Result<()> {
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
            NetworkRequest::CreateFile{path} => {
                w.write_string(path)?;
            },
        }
        Ok(())
    }
}

impl NetworkResponse {

    fn message_id(&self) -> u8 {
        match self {
            NetworkResponse::ListFiles{..} => LIST_FILES,
            NetworkResponse::GetFileData{..} => GET_FILE_DATA,
            NetworkResponse::CreateFile{..} => CREATE_FILE,
        }
    }

    pub fn read<T: ReadExt>(r: &mut T) -> io::Result<NetworkResponse> {
        let message_id = r.read_u8()?;
        match message_id {
            LIST_FILES => {
                let files = r.read_string_array()?;
                Ok(NetworkResponse::ListFiles{files})
            },
            GET_FILE_DATA => {
                let from_revision = r.read_length()?;
                let to_revision = r.read_length()?;
                let data = r.read_bytes()?;
                Ok(NetworkResponse::GetFileData{from_revision, to_revision, data})
            },
            CREATE_FILE => {
                let result = r.read_u8()?;
                Ok(NetworkResponse::CreateFile{result: if result == 0 { Ok(()) } else { Err(r.read_string()?) }})
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported message id {}", message_id))),
        }
    }

    pub fn write<T: WriteExt>(&self, w: &mut T) -> io::Result<()> {
        w.write_u8(self.message_id())?;
        match self {
            NetworkResponse::ListFiles { files } => {
                w.write_string_array(files)
            },
            NetworkResponse::GetFileData { from_revision, to_revision, data } => {
                w.write_length(*from_revision)?;
                w.write_length(*to_revision)?;
                w.write_bytes(data)
            },
            NetworkResponse::CreateFile { result } => {
                if let Err(e) = result {
                    w.write_u8(1)?;
                    w.write_string(e)
                } else {
                    w.write_u8(0)
                }
            }
        }
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
            NetworkRequest::CreateFile { path } => {
                write!(f, "CreateFile: {}", path)
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
            NetworkResponse::CreateFile { result } => {
                match result {
                    Ok(()) => write!(f, "CreateFile: OK"),
                    Err(e) => write!(f, "CreateFile: {}", e),
                }
            },
        }
    }
}