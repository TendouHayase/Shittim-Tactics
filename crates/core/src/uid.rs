#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(u64);

impl UID {
    pub fn new(uid: u64) -> Self {
        UID(uid)
    }
}
