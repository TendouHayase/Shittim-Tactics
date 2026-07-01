use core::{terrains::Terrain, types::AttackType};

use error::Error;

use crate::difficulty::Difficulty;

pub mod binah;

pub trait Boss {
    /// Generates bosses with stats tailored to the difficulty, attack type, and terrain.
    fn from_file(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    fn hp(&self) -> u64;

    fn status(&self) -> &Self;

    fn mut_status(&mut self) -> &mut Self;
}
