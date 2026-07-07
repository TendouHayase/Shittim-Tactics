use std::{collections::LinkedList, rc::Rc};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    Position,
    base::BaseStats,
    damage::{Damage, DamageCache},
    difficulty::Difficulty,
    skill::{Effect, Skill, SkillEffect},
    terrains::Terrain,
    types::AttackType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct BossStats {
    pub name: String,
    pub id: u32,
    pub base_stats: BaseStats,
    pub terrain: Terrain,
    pub groggy_gauge: u64,
    pub groggy_duration: u8,
}

#[derive(Debug, Clone)]
pub struct Boss<T: BossTrait> {
    pub stats: BossStats,
    pub other_stats: T,
    pub skills: Rc<Vec<Rc<dyn Skill>>>,
}

impl<T: BossTrait> PartialEq for Boss<T> {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

impl<T: BossTrait> Eq for Boss<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossState<'a, T: BossTrait> {
    pub boss: &'a Boss<T>,
    /// These are the student's coordinates.
    pub coordinate: Position,

    /// These are the cooldowns for EX, Basic, Enhanced, and Sub Skills.
    pub cooldowns: Vec<u32>,

    pub effects: LinkedList<Effect>,

    pub accumulated_damage: Vec<Damage>,
    pub accumulated_damage_cache: DamageCache,
}

pub trait BossTrait {
    /// Generates bosses with stats tailored to the difficulty, attack type, and terrain.
    fn from_file(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    fn stats(&self) -> &BossStats;
}
