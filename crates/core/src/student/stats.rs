use error::Error;
use serde::{Deserialize, Serialize};

use crate::{
    base::BaseStats,
    constants::MAX_SKILL_LEVEL,
    stat::{StatKind, StatValueKind},
    student::{file::StudentFile, spec::StudentSpec},
    table::{gear::GearTable, level::calcul_stat},
    terrains::{Terrain, TerrainCombatPowerState},
    utils::Ratio,
};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone)]
pub struct StudentStats {
    pub student_stats: StudentSpec,
    pub base_stats: BaseStats,
}

impl PartialEq for StudentStats {
    fn eq(&self, other: &Self) -> bool {
        self.student_stats.uid == other.student_stats.uid
    }
}

impl Eq for StudentStats {}

impl Hash for StudentStats {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.student_stats.uid.hash(state);
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
    pub curve: [Ratio; MAX_SKILL_LEVEL],
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

/// Level stats folded together with the gear, in the order the game applies them: every flat
/// increase is summed, then the summed rates multiply once.
pub(crate) fn build_stats(
    file: &StudentFile,
    spec: &StudentSpec,
    gears: &GearTable,
) -> Result<BaseStats, Error> {
    let raw = calcul_stat(
        spec.star,
        spec.level,
        spec.weapon_level,
        spec.talent_levels,
        file.unique_weapon.clone(),
        file.level_stats.clone(),
    )
    .ok_or_else(|| {
        Error::InvalidData(format!(
            "level {} is outside 1..=90 for {}",
            spec.level, spec.name
        ))
    })?;

    let mut mods: HashMap<StatKind, (f64, f64)> = HashMap::new();
    for i in 0..spec.gear_kinds.len() {
        let Some(stats) = gears.stats(
            spec.gear_kinds[i],
            spec.gear_tiers[i] as usize,
            spec.gear_levels[i] as usize,
        ) else {
            continue;
        };

        for stat in stats {
            let entry = mods.entry(stat.stat).or_insert((0.0, 0.0));
            match stat.kind {
                StatValueKind::Amount => entry.0 += stat.value.0,
                StatValueKind::Scale => entry.1 += stat.value.0,
            }
        }
    }

    if spec.weapon_star >= 2 {
        let star2_option = &file.unique_weapon.star2_option;
        let entry = mods.entry(star2_option.stat).or_insert((0.0, 0.0));

        match star2_option.kind {
            StatValueKind::Amount => {
                entry.0 += star2_option.curve[spec.skill_levels[2] as usize - 1].to_f64()
            }
            StatValueKind::Scale => {
                entry.1 += star2_option.curve[spec.skill_levels[2] as usize - 1].to_f64()
            }
        }
    }

    let fold = |mods: &mut HashMap<StatKind, (f64, f64)>, stat, base: f64| {
        let (amount, scale) = mods.remove(&stat).unwrap_or((0.0, 0.0));
        ((base + amount) * (1.0 + scale / 100.0)).round()
    };

    let mut base_stats = file.lvl1_stats;
    base_stats.level = spec.level;
    base_stats.hp = fold(&mut mods, StatKind::Hp, raw.hp) as u64;
    base_stats.atk = fold(&mut mods, StatKind::Atk, raw.atk) as u32;
    base_stats.def = fold(&mut mods, StatKind::Def, raw.def) as u32;
    base_stats.healing = fold(&mut mods, StatKind::Healing, raw.healing) as u32;

    for (stat, (amount, scale)) in mods {
        base_stats = base_stats.apply_stat(stat, amount, 1.0 + scale / 100.0);
    }

    Ok(base_stats)
}
