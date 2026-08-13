use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomPinned};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    base::BaseStats,
    character::CharacterOps,
    difficulty::Difficulty,
    locale::LocalizedName,
    skill::Skill,
    skills::{
        binah::{BinahAtsilutsLight, BinahFiresofSeverity, BinahPurifyingStorm},
        goz::{GozMagicalCoinHat, GozNowYouSeeUs, GozThreeLightMonte},
        perorodzilla::{
            PerorodzillaAbsorbMinion, PerorodzillaAquaBall, PerorodzillaBurningPerorodzilla,
            PerorodzillaHyperSpiralGlareBeam, PerorodzillaSummonMinion,
            PerorodzillaWhiteHotHeatVision,
        },
    },
    terrains::Terrain,
    types::ArmorType,
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

#[derive(Debug, TypedBuilder)]
pub struct Boss {
    pub stats: BossStats,
    pub skills: Vec<Skill>,

    _pin: PhantomPinned,
}

/// Top level of `data/bosses/<boss>.json`.
///
/// Armor type keys differ per boss, so every remaining key is swept up. Any top-level key that
/// is not an armor type, such as `skills`, must therefore be declared as a field here; leaving
/// one out surfaces as an `ArmorType` parse failure.
#[derive(Debug, Deserialize)]
struct BossFile {
    id: u32,
    name: LocalizedName,
    skills: serde_json::Value,

    #[serde(flatten)]
    by_armor: HashMap<ArmorType, HashMap<Difficulty, DifficultyEntry>>,
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

impl CharacterOps for Boss {
    fn id(&self) -> u32 {
        self.stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skill_list(&self) -> &[Skill] {
        &self.skills
    }
}

impl Boss {
    /// Returns a `Box` because the skills point back at their boss through `NonNull<Boss>`.
    /// Returning by value would move the boss out and leave every one of those pointers dangling.
    pub fn from_file(
        kind: BossKind,
        path: &str,
        armor_type: ArmorType,
        difficulty: Difficulty,
        terrain: Terrain,
        skill_mask_offset: usize,
    ) -> Result<Box<Self>, Error> {
        let file: BossFile = parsing_json::read_json(path)?;

        let entry = file
            .by_armor
            .get(&armor_type)
            .ok_or_else(|| {
                Error::InvalidData(format!("can not find armor type {armor_type:?} in {path}"))
            })?
            .get(&difficulty)
            .ok_or_else(|| {
                Error::InvalidData(format!("can not find difficulty {difficulty:?} in {path}"))
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

        let mut boss = Box::new(
            Boss::builder()
                .stats(stats)
                ._pin(PhantomPinned)
                .skills(Vec::new())
                .build(),
        );

        boss.skills = build_skills(&boss, kind, file.skills, skill_mask_offset)?;

        Ok(boss)
    }
}

/// Which boss is being loaded. The json `id` is a value the game assigns, unknown to this code,
/// so it cannot serve as the key that selects a skill list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossKind {
    Binah,
    Goz,
    Perorodzilla,
}

/// The one place [`BossKind`] is tied to a Rust skill list. Skills are code and cannot be moved
/// into data.
///
/// Bosses whose numbers are not transcribed yet ignore `skills` and build from
/// `Params::of(difficulty)`.
fn build_skills(
    boss: &Boss,
    kind: BossKind,
    skills: serde_json::Value,
    offset: usize,
) -> Result<Vec<Skill>, Error> {
    use crate::skills::binah::params::RawSkills as BinahSkills;

    let difficulty = boss.stats.difficulty;

    let skills = match kind {
        BossKind::Binah => {
            let raw: BinahSkills = serde_json::from_value(skills)?;

            vec![
                Skill::BinahAtsilutsLight(BinahAtsilutsLight::new(
                    boss,
                    offset,
                    raw.atsiluts_light.name.get().to_string(),
                    raw.atsiluts_light.pick(difficulty),
                )),
                Skill::BinahFiresofSeverity(BinahFiresofSeverity::new(
                    boss,
                    offset + 1,
                    raw.fires_of_severity.name.get().to_string(),
                    raw.fires_of_severity.pick(difficulty),
                )),
                Skill::BinahPurifyingStorm(BinahPurifyingStorm::new(
                    boss,
                    offset + 2,
                    raw.purifying_storm.name.get().to_string(),
                    raw.purifying_storm.pick(difficulty),
                )),
            ]
        }

        BossKind::Goz => vec![
            Skill::GozMagicalCoinHat(GozMagicalCoinHat::new(boss, offset)),
            Skill::GozNowYouSeeUs(GozNowYouSeeUs::new(boss, offset + 1)),
            Skill::GozThreeLightMonte(GozThreeLightMonte::new(boss, offset + 2)),
        ],

        BossKind::Perorodzilla => vec![
            Skill::PerorodzillaWhiteHotHeatVision(PerorodzillaWhiteHotHeatVision::new(
                boss, offset,
            )),
            Skill::PerorodzillaAquaBall(PerorodzillaAquaBall::new(boss, offset + 1)),
            Skill::PerorodzillaSummonMinion(PerorodzillaSummonMinion::new(boss, offset + 2)),
            Skill::PerorodzillaAbsorbMinion(PerorodzillaAbsorbMinion::new(boss, offset + 3)),
            Skill::PerorodzillaHyperSpiralGlareBeam(PerorodzillaHyperSpiralGlareBeam::new(
                boss,
                offset + 4,
            )),
            Skill::PerorodzillaBurningPerorodzilla(PerorodzillaBurningPerorodzilla::new(
                boss,
                offset + 5,
            )),
        ],
    };

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{SkillMeta, SkillOps};

    /// Relative to the crate root, not the workspace root.
    const BINAH: &str = "../../data/bosses/binah.json";

    #[test]
    fn load_binah_lunatic() {
        let boss = Boss::from_file(
            BossKind::Binah,
            BINAH,
            ArmorType::Heavy,
            Difficulty::Lunatic,
            Terrain::Outdoor,
            0,
        )
        .expect("failed to load binah");

        assert_eq!(boss.stats().hp, 50_000_000);
        assert_eq!(boss.stats.groggy_gauge, 10_000_000);
        assert_eq!(boss.skill_list().len(), 3);

        // 정화의 폭풍만 코스트가 있음.
        assert_eq!(boss.skill_list()[2].cost(), 3);
        assert_eq!(boss.skill_list()[2].duration(), 30);
    }

    /// Catches an off-by-one index into the per-difficulty arrays.
    #[test]
    fn skill_params_follow_difficulty() {
        let effects_of = |difficulty| {
            Boss::from_file(
                BossKind::Binah,
                BINAH,
                ArmorType::Heavy,
                difficulty,
                Terrain::Outdoor,
                0,
            )
            .expect("failed to load binah")
            .skill_list()[0]
                .skill_effects()
        };

        let normal = effects_of(Difficulty::Normal);
        let lunatic = effects_of(Difficulty::Lunatic);

        assert_ne!(normal[0].targets, lunatic[0].targets);
        assert_eq!(
            normal[1].timing,
            crate::effect::EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 450,
            }
        );
        assert_eq!(
            lunatic[1].timing,
            crate::effect::EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 3600,
            }
        );
    }
}
