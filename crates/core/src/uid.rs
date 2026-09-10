#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(u64);

impl Uid {
    pub fn new(uid: u64) -> Self {
        Uid(uid)
    }
}
