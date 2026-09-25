use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{base::BaseStats, difficulty::Difficulty, terrains::Terrain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct BossStats {
    pub name: String,
    pub id: u32,
    pub base_stats: BaseStats,
    pub terrain: Terrain,
    pub groggy_gauge: u64,
    pub groggy_duration: u8,
    pub difficulty: Difficulty,
    pub phase_switching_hp: [u64; 3],
}
