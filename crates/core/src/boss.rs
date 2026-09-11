use std::{collections::HashMap, fmt::Debug, hash::Hash};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    base::BaseStats,
    character::Character,
    difficulty::Difficulty,
    locale::LocalizedName,
    skill::Skill,
    terrains::Terrain,
    types::ArmorType,
    uid::Uid,
};

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

#[derive(Debug)]
pub struct Boss {
    pub stats: BossStats,
    pub skills: Vec<Box<dyn Skill>>,
}

/// Top level of `data/bosses/<boss>.json`.
///
/// Armor type keys differ per boss, so every remaining key is swept up. Any top-level key that
/// is not an armor type, such as `skills`, must therefore be declared as a field here; leaving
/// one out surfaces as an `ArmorType` parse failure.
#[derive(Debug, Deserialize)]
pub struct BossFile {
    pub id: u32,
    pub name: LocalizedName,
    pub skills: serde_json::Value,

    #[serde(flatten)]
    by_armor: HashMap<ArmorType, HashMap<Difficulty, DifficultyEntry>>,
}

impl BossFile {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        Ok(parsing_json::read_json(path)?)
    }
}

#[derive(Debug, Deserialize)]
struct DifficultyEntry {
    #[serde(flatten)]
    stats: BaseStats,
    groggy_gauge: u64,
    groggy_duration: u8,
    phase_switching_hp: [u64; 3],
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

impl Character for Boss {
    fn id(&self) -> Uid {
        Uid::new(self.stats.id as u64)
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skills(&self) -> &[Box<dyn Skill>] {
        &self.skills
    }
}

impl Boss {
    pub fn new(
        file: &BossFile,
        armor_type: ArmorType,
        difficulty: Difficulty,
        terrain: Terrain,
        skills: Vec<Box<dyn Skill>>,
    ) -> Result<Self, Error> {
        let entry = file
            .by_armor
            .get(&armor_type)
            .ok_or_else(|| {
                Error::InvalidData(format!(
                    "can not find armor type {armor_type:?} for boss {}",
                    file.id
                ))
            })?
            .get(&difficulty)
            .ok_or_else(|| {
                Error::InvalidData(format!(
                    "can not find difficulty {difficulty:?} for boss {}",
                    file.id
                ))
            })?;

        let stats = BossStats::builder()
            .name(file.name.get().to_string())
            .id(file.id)
            .base_stats(entry.stats)
            .terrain(terrain)
            .groggy_gauge(entry.groggy_gauge)
            .groggy_duration(entry.groggy_duration)
            .difficulty(difficulty)
            .phase_switching_hp(entry.phase_switching_hp)
            .build();

        Ok(Boss { stats, skills })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative to the crate root, not the workspace root.
    const BINAH: &str = "../../data/bosses/binah.json";

    #[test]
    fn load_binah_lunatic() {
        let file = BossFile::from_file(BINAH).expect("failed to load binah");
        let boss = Boss::new(
            &file,
            ArmorType::Heavy,
            Difficulty::Lunatic,
            Terrain::Outdoor,
            Vec::new(),
        )
        .expect("failed to build binah");

        assert_eq!(boss.stats().hp, 50_000_000);
        assert_eq!(boss.stats.groggy_gauge, 10_000_000);
    }
}
