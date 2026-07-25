use std::{fmt::Debug, hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{base::BaseStats, character::Character, skill::Skill, terrains::Terrain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct BossStats {
    pub name: String,
    pub id: u32,
    pub base_stats: BaseStats,
    pub terrain: Terrain,
    pub groggy_gauge: u64,
    pub groggy_duration: u8,
}

#[derive(Debug)]
pub struct Boss {
    pub stats: BossStats,
    pub skills: Vec<Skill>,
}

impl PartialEq for Boss {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

impl Eq for Boss {}

impl Hash for Boss {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.id.hash(state);
    }
}

impl Boss {
    pub fn id(&self) -> u32 {
        self.stats.id
    }

    pub fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    pub fn skill_list(&self) -> &[Skill] {
        &self.skills
    }
}
