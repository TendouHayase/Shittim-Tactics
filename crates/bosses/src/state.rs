use core::{skill::Skill, state::State, student::Student};

use crate::boss::Boss;

pub struct CommonState<S: State> {
    pub students: Vec<Student>,
    pub cost: i8,
    pub boss: dyn Boss<State = S, Skill = dyn Skill>,
}
