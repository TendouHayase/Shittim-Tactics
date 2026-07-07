use crate::{
    boss::{BossState, BossTrait},
    student::StudentState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateData<'a, const STUDENT_COUNT: usize, T: BossTrait> {
    pub students: [StudentState<'a>; STUDENT_COUNT],
    pub cost: i8,
    pub boss: BossState<'a, T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State<'a, T: BossTrait> {
    Assault(StateData<'a, 6, T>),
    RestrictionRelease(StateData<'a, 10, T>),
}

impl<'a, T: BossTrait + Clone> State<'a, T> {
    pub fn students(&self) -> &[StudentState<'a>] {
        match self {
            State::Assault(inner) => &inner.students,
            State::RestrictionRelease(inner) => &inner.students,
        }
    }

    pub fn boss(&self) -> BossState<'a, T> {
        match self {
            State::Assault(inner) => inner.boss.clone(),
            State::RestrictionRelease(inner) => inner.boss.clone(),
        }
    }
}
