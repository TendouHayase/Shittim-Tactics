use std::fmt::Debug;

use crate::uid::Uid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Character {
    Student(Uid),
    Boss(Uid),
}

impl Character {
    pub fn is_boss(&self) -> bool {
        match self {
            Self::Boss(_) => true,
            _ => false,
        }
    }

    pub fn is_student(&self) -> bool {
        match self {
            Self::Student(_) => true,
            _ => false,
        }
    }
}
