use uuid::Uuid;

pub fn shorten_uuid(uuid: &Uuid) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}