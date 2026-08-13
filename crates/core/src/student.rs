use std::{fmt::Debug, hash::Hash, marker::PhantomPinned};

use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{
    base::BaseStats,
    character::CharacterOps,
    constants::MAX_SKILL_LEVEL,
    locale::LocalizedName,
    skill::Skill,
    stat::{StatKind, StatValueKind},
    table::gear::GearKind,
    terrains::{Terrain, TerrainCombatPower, TerrainCombatPowerState},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,

    /// The elements in this array represent the levels of the following skills.
    /// Ex skill, Basic Skill, Enhanced Skill, Sub Skill
    pub skill_levels: [u8; 4],
    pub weapon_level: u8,
    pub bond_level: u8,

    /// Affinity Level of the Separated Character
    pub alter_bond_levels: Vec<u8>,

    pub gear_kinds: [GearKind; 3],
    pub gear_tiers: [u8; 3],
    pub gear_levels: [u8; 3],

    /// Each element in this array represents the following.
    /// Max HP Talent level, ATK Talent Level, Healing Talent Level
    pub talent_levels: [u8; 3],

    pub unique_item_level: Option<u8>,
}

/// Top level of `data/students/<student>.json`.
///
/// Growth from level, star tier and talent follows shared formulas and is not stored here. This
/// holds only the per-student values those formulas take as input.
#[derive(Debug, Clone, Deserialize)]
pub struct StudentFile {
    pub id: u32,
    pub name: LocalizedName,
    pub terrain_adaptation: TerrainCombatPower,

    /// The three gear kinds this student can equip. Their numbers live in the gear data.
    pub gear_slots: [GearKind; 3],

    /// Level 1 stats. `level` is a runtime value, absent from the file and left at 0.
    pub lvl1_stats: BaseStats,

    /// Level 1 and level 90 observations for each star tier.
    pub level_stats: StarCurves,
    pub unique_weapon: UniqueWeapon,

    /// Per-skill numbers. The fields differ by student, so this is left unopened here and read
    /// by that student's own crate into its `params` type.
    pub skills: serde_json::Value,
}

impl StudentFile {
    pub fn from_file(path: &str) -> Result<Self, error::Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct StarCurves {
    pub hp: StarValue<u64>,
    pub atk: StarValue<u32>,
    pub def: StarValue<u32>,
    pub healing: StarValue<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct StarValue<T> {
    pub lvl1: [T; 5],
    pub lvl90: [T; 5],
}

/// The four stats that grow with level. The rest do not, and `lvl1_stats` carries them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct LevelStats {
    pub hp: u64,
    pub atk: u32,
    pub def: u32,
    pub healing: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawStats {
    pub hp: f64,
    pub atk: f64,
    pub def: f64,
    pub healing: f64,
}

/// The student's unique weapon.
#[derive(Debug, Clone, Deserialize)]
pub struct UniqueWeapon {
    /// Values at levels 1, 30, 40, 50 and 60, which are the caps of each star tier.
    pub hp: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// Values at levels 1, 30, 40, 50 and 60, which are the caps of each star tier.
    pub atk: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// Stat added at weapon star 2.
    pub star2_option: EnhancedSkillPlus,

    /// Terrain adaptation raised at weapon star 3, and the value it reaches.
    pub star3_option: (Terrain, TerrainCombatPowerState),

    pub star4_option: UniqueWeapon4StarOption,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnhancedSkillPlus {
    pub stat: StatKind,
    pub kind: StatValueKind,
    pub curve: [u32; MAX_SKILL_LEVEL],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UniqueWeapon4StarOption {
    /// Maximum cost up by 0.5.
    MaxCostUp,
    ExplosiveEffectiveness,
    PiercingEffectiveness,
    CorrosiveEffectiveness,
    MysticEffectiveness,
    SonicEffectiveness,
}

impl UniqueWeapon {
    pub const MAX_STAR: u8 = 4;
}

#[derive(Debug, Clone)]
pub struct StudentStats {
    pub student_stats: StudentSpec,
    pub base_stats: BaseStats,
}

impl PartialEq for StudentStats {
    fn eq(&self, other: &Self) -> bool {
        self.student_stats.id == other.student_stats.id
    }
}

impl Eq for StudentStats {}

impl Hash for StudentStats {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.student_stats.id.hash(state);
    }
}

#[derive(Debug)]
pub struct Student {
    pub stats: StudentStats,

    /// Ex, Basic and Sub. The enhanced skill is always a stat increase, so it is folded into
    /// [`StudentStats::base_stats`] instead of being a skill.
    pub skills: Vec<Skill>,

    _pin: PhantomPinned,
}

impl Student {
    pub fn from_file() -> Result<Box<Self>, error::Error> {
        todo!()
    }
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

impl CharacterOps for Student {
    fn id(&self) -> u32 {
        self.stats.student_stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skill_list(&self) -> &[Skill] {
        &self.skills
    }
}

impl Hash for Student {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.hash(state);
    }
}

impl Eq for Student {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudentKind {
    Kei,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrains::{Terrain, TerrainCombatPowerState};
    use crate::types::AttackType;

    #[test]
    fn load_kei() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");

        assert_eq!(file.id, 10135);
        assert_eq!(file.lvl1_stats.attack_type, AttackType::Mystic);
        assert_eq!(file.lvl1_stats.level, 0);
        assert_eq!(file.gear_slots[0], GearKind::Shoes);
    }

    #[test]
    fn unique_weapon_promotes_the_top_terrain() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");
        let mut adaptation = file.terrain_adaptation;

        assert_eq!(adaptation.get(Terrain::Street), TerrainCombatPowerState::S);
        adaptation.promote_best();

        assert_eq!(adaptation.get(Terrain::Street), TerrainCombatPowerState::SS);
        assert_eq!(adaptation.get(Terrain::Indoor), TerrainCombatPowerState::A);
    }
}
