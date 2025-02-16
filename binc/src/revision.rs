use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::readwrite::ReadExt;
use crate::readwrite::WriteExt;
use chrono::Utc;
use whoami::username;
use crate::change::Change;
use crate::changes::Changes;

pub struct Revision {
    pub changes: Vec<Change>,
    pub id: Uuid,
    pub uuid_of_parents: Vec<Uuid>,
    pub date: String,
    pub user_name: String,
    pub message: String,
    pub tags: Vec<String>,
}

impl From<Changes> for Revision {
    fn from(changes: Changes) -> Self {
        let mut revision = Revision::new(None);
        revision.changes = changes.changes;
        revision
    }
}

impl Revision {

    pub const CHANGE_LIST_ID: u32 = 0x43686E67;

    pub fn new(last_revision: Option<&Revision>) -> Revision {
        let mut revision = Revision {
            changes: vec![],
            id: Uuid::new_v4(),
            uuid_of_parents: vec![],
            date: Utc::now().to_rfc3339(),
            user_name: username(),
            message: String::new(),
            tags: vec![],
        };
        if let Some(last_revision) = last_revision {
            revision.uuid_of_parents.push(last_revision.id);
        }
        revision
    }

    pub fn add_change(&mut self, change: Change)
    {
        self.changes.push(change);
    }

    pub fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        let mut bytes: Vec<u8> = vec![];
        self.write_content(&mut bytes)?;
        w.write_all(&bytes);
        let hash = blake3::hash(&bytes);
        w.write_hash(&hash)?;

        // TODO hash should include has of parent revisions
        // for now hash is merely used as a placeholder
        Ok(())
    }

    fn write_content(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_u32(Self::CHANGE_LIST_ID)?;
        w.write_uuid(&self.id)?;
        w.write_uuid_array(&self.uuid_of_parents)?;
        w.write_string(&self.date)?;
        w.write_string(&self.user_name)?;
        w.write_string(&self.message)?;
        w.write_string_array(&self.tags)?;

        w.write_length(self.changes.len() as u64)?;
        for change in &self.changes {
            w.write_length(change.change_type())?;
            let mut temp: Vec<u8> = vec![];
            change.write(&mut temp)?;
            w.write_length(temp.len() as u64)?;
            w.write_all(&temp)?
        }
        Ok(())
    }

    pub fn read(mut r: &mut dyn Read) -> io::Result<Revision> {

        let result = Self::read_content(r);
        let hash = r.read_hash()?;
        // TODO verify hash

        result
    }

    fn read_content(mut r: &mut dyn Read) -> io::Result<Revision> {
        let id = r.read_u32()?;
        if id != Self::CHANGE_LIST_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let mut revision = Revision::new(None);
        revision.id = r.read_uuid()?;
        revision.uuid_of_parents = r.read_uuid_array()?;
        revision.date = r.read_string()?;
        revision.user_name = r.read_string()?;
        revision.message = r.read_string()?;
        revision.tags = r.read_string_array()?;

        let count = r.read_length()?;

        for _ in 0..count {
            let change = Change::read(r)?;
            revision.changes.push(change);
        }

        Ok(revision)
    }
}

/*
struct HashReader {
    r: dyn &Read,
    bytes: Vec<u8>,
}

impl Read for HashReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = self.r.read(buf)?;
        self.bytes.extend_from_slice(&buf[..result.clone()]);
        result
    }

    fn get_hash(&self) -> Hash {
        hash(&self.bytes)
    }
}
*/