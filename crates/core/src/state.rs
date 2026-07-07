use crate::{
    boss::{Boss, BossStats, BossTrait},
    student::Student,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateData<const STUDENT_COUNT: usize, T: BossTrait> {
    pub students: [Student; STUDENT_COUNT],
    pub cost: i8,
    pub boss: Boss<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State<T: BossTrait> {
    Assault(StateData<6, T>),
    RestrictionRelease(StateData<10, T>),
}

impl<T: BossTrait + Clone> State<T> {
    pub fn students(&self) -> &[Student] {
        match self {
            State::Assault(inner) => &inner.students,
            State::RestrictionRelease(inner) => &inner.students,
        }
    }

    pub fn boss(&self) -> Boss<T> {
        match self {
            State::Assault(inner) => inner.boss.clone(),
            State::RestrictionRelease(inner) => inner.boss.clone(),
        }
    }
}
