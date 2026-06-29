use core::{actions::Action, skill::Skill, state::State, terrains::Terrain, types::AttackType};

use error::Error;

use crate::difficulty::Difficulty;

pub mod binah;

pub trait Boss: Send + Sync {
    type State: State;
    type Skill: Skill;

    /// Generates bosses with stats tailored to the difficulty, attack type, and terrain.
    fn new(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    /// Takes a state and an action and returns a state.
    fn step(&self, state: &Self::State, action: Action<Self::Skill>) -> Self::State;
}
