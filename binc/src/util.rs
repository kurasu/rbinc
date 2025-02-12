use uuid::Uuid;
use crate::id::NodeId;

pub fn shorten_uuid(uuid: &NodeId) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}