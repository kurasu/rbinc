mod revision;
mod document;
mod io;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::revision::*;
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn changes1() {
        let a = Change::AddNode(Uuid::new_v4());
        assert_eq!(get_change_id(a), 0x1)
    }
}
