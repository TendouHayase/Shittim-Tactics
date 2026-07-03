use std::hash::Hash;

use crate::{boss::BossStats, student::StudentStat};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateData<const STUDENT_COUNT: usize> {
    pub students: [StudentStat; STUDENT_COUNT],
    pub cost: i8,
    pub boss: BossStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    Assault(StateData<6>),
    RestrictionRelease(StateData<10>),
}
