use uuid::Uuid;
use crate::id::Id;

pub fn shorten_uuid(uuid: &Id) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}