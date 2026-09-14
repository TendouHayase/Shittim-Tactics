use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Uid(u64);

impl Uid {
    pub fn new(uid: u64) -> Self {
        Uid(uid)
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillUid(Uid, usize);

impl SkillUid {
    pub fn owner(&self) -> Uid {
        self.0
    }

    /// Global skill index
    pub fn skill_index(&self) -> usize {
        self.1
    }
}

impl SkillUid {
    pub fn new(owner: Uid, index: usize) -> Self {
        SkillUid(owner, index)
    }
}
