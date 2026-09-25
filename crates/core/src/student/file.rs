use serde::Deserialize;

use crate::{
    base::BaseStats,
    locale::LocalizedName,
    student::stats::{StarCurves, UniqueWeapon},
    table::gear::GearKind,
    terrains::TerrainCombatPower,
};

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
