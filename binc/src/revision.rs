use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use uuid::Uuid;
use crate::iowrappers::ReadExt;
use crate::iowrappers::WriteExt;
use chrono::Utc;
use whoami::username;
use crate::document::Node;


pub trait Change {
    fn change_type(&self) -> u64;
    fn write(&self, w: &mut dyn Write) -> io::Result<()>;
    fn apply(&self, nodes: &mut HashMap<Uuid, crate::document::Node>);
}

pub struct Revision {
    pub(crate) changes: Vec<Box<dyn Change>>,
    pub(crate) id: Uuid,
    pub(crate) uuid_of_parents: Vec<Uuid>,
    pub(crate) date: String,
    pub(crate) user_name: String,
    pub(crate) message: String,
    pub(crate) tags: Vec<String>,
}

impl Revision {

    pub const CHANGE_LIST_ID: u32 = 0x42494E43;

    pub fn new() -> Revision {
        Revision{
            changes: vec![],
            id: Uuid::new_v4(),
            uuid_of_parents: vec![],
            date: Utc::now().to_rfc3339(),
            user_name: username(),
            message: String::new(),
            tags: vec![],
        }
    }

    pub fn add_change(&mut self, change: Box<dyn Change>)
    {
        self.changes.push(change);
    }

    pub fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        w.write_u32(Self::CHANGE_LIST_ID)?;
        w.write_uuid(&self.id)?;
        w.write_uuid_array(&self.uuid_of_parents)?;
        w.write_string(&self.date)?;
        w.write_string(&self.user_name)?;
        w.write_string(&self.message)?;
        w.write_string_array(&self.tags)?;

        w.write_length(self.changes.len() as u64)?;
        for change in &self.changes {
            change.write(w)?;
        }
        Ok(())
    }

    pub fn read(mut r: &mut dyn Read) -> io::Result<Revision> {
        let id = r.read_u32()?;
        if id != Self::CHANGE_LIST_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let mut revision = Revision::new();
        revision.id = r.read_uuid()?;
        revision.uuid_of_parents = r.read_uuid_array()?;
        revision.date = r.read_string()?;
        revision.user_name = r.read_string()?;
        revision.message = r.read_string()?;
        revision.tags = r.read_string_array()?;

        let count = r.read_length()?;

        for _ in 0..count {
            let change = crate::changes::read_change(r)?;
            revision.changes.push(change);
        }

        Ok(revision)
    }
}
