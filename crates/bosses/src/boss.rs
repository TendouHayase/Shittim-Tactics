use core::{actions::Action, skill::Skill, state::State, terrains::Terrain, types::AttackType};

use error::Error;

use crate::difficulty::Difficulty;

pub mod binah;

pub trait Boss: Send + Sync {
    type State: State;

    fn new(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    fn step(&self, state: &Self::State, action: Action<impl Skill>) -> Self::State;
}
