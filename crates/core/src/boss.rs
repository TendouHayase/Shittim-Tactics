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

#[derive(Debug, Clone)]
pub struct Boss<T: PartialEq + Send + Sync> {
    pub stats: BossStats,
    pub other_stats: T,
    pub skills: Vec<Arc<dyn Skill>>,
}

impl<T: PartialEq + Send + Sync> PartialEq for Boss<T> {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats && self.other_stats == other.other_stats
    }
}

impl<T: PartialEq + Send + Sync> Eq for Boss<T> {}

impl<T: PartialEq + Send + Sync> Hash for Boss<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.id.hash(state);
    }
}

impl<T: PartialEq + Send + Sync + Debug> Character for Boss<T> {
    fn id(&self) -> u32 {
        self.stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skill_list(&self) -> &Vec<Arc<dyn Skill>> {
        &self.skills
    }
}
