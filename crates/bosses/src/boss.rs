use core::{actions::Action, skill::Skill, state::State};

use crate::difficulty::Difficulty;

pub mod binah;

pub trait Boss: Send + Sync {
    type State: State;

    fn new(difficulty: &Difficulty) -> Self;

    fn step(&self, state: &Self::State, action: Action<impl Skill>) -> Self::State;
}
