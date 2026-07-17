use std::{collections::LinkedList, hash::Hash, rc::Rc, sync::Arc};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    base::BaseStats, character::Character, damage::cache::DamageCache, skill::Skill,
    terrains::Terrain,
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
pub struct Boss<T: BossBehavior> {
    pub stats: BossStats,
    pub other_stats: T,
    pub skills: Vec<Arc<dyn Skill>>,
}

impl<T: BossBehavior + PartialEq> PartialEq for Boss<T> {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats && self.other_stats == other.other_stats
    }
}

impl<T: BossBehavior + PartialEq> Eq for Boss<T> {}

impl<T: Character + BossBehavior> Hash for Boss<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.id.hash(state);
    }
}

impl<T: Character + BossBehavior> Character for Boss<T> {
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

pub trait BossBehavior: Character {
    fn groggy_gauge(&self) -> u64;
    fn groggy_duration(&self) -> u8;
    fn terrain(&self) -> Terrain;
}
