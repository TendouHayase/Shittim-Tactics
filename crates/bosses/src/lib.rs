pub mod binah;
pub mod goz;
pub mod macros;
pub mod perorodzilla;
pub mod states;

use crate::macros::SkillNumbers;
use binah::skills::{
    BinahAtsilutsLight, BinahFiresofSeverity, BinahPurifyingStorm, params::RawSkills,
};
use core::{
    boss::{Boss, BossFile},
    difficulty::Difficulty,
    skill::Skill,
    terrains::Terrain,
    types::ArmorType,
    uid::Uid,
};
use error::Error;
use goz::skills::{GozMagicalCoinHat, GozNowYouSeeUs, GozThreeLightMonte, params as goz_params};
use perorodzilla::skills::{
    PerorodzillaAbsorbMinion, PerorodzillaAquaBall, PerorodzillaBurningPerorodzilla,
    PerorodzillaHyperSpiralGlareBeam, PerorodzillaSummonMinion, PerorodzillaWhiteHotHeatVision,
    params as pero_params,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossKind {
    Binah,
    Goz,
    Perorodzilla,
}

pub fn load(
    kind: BossKind,
    path: &str,
    armor_type: ArmorType,
    difficulty: Difficulty,
    terrain: Terrain,
    skill_offset: usize,
) -> Result<Boss, Error> {
    let file = BossFile::from_file(path)?;
    let skills = build_skills(kind, &file, difficulty, skill_offset)?;

    Boss::new(&file, armor_type, difficulty, terrain, skills)
}

fn build_skills(
    kind: BossKind,
    file: &BossFile,
    difficulty: Difficulty,
    offset: usize,
) -> Result<Vec<Box<dyn Skill>>, Error> {
    let owner = Uid::new(file.id as u64);
    let name = || file.name.get().to_string();

    Ok(match kind {
        BossKind::Binah => {
            let raw = RawSkills::deserialize(&file.skills)?;

            let atsiluts_light = raw.atsiluts_light.pick(difficulty);
            let fires_of_severity = raw.fires_of_severity.pick(difficulty);
            let purifying_storm = raw.purifying_storm.pick(difficulty);

            vec![
                Box::new(BinahAtsilutsLight::new(
                    owner,
                    offset,
                    raw.atsiluts_light.name.get().to_string(),
                    SkillNumbers::of(&atsiluts_light),
                    atsiluts_light,
                )),
                Box::new(BinahFiresofSeverity::new(
                    owner,
                    offset + 1,
                    raw.fires_of_severity.name.get().to_string(),
                    SkillNumbers::of(&fires_of_severity),
                    fires_of_severity,
                )),
                Box::new(BinahPurifyingStorm::new(
                    owner,
                    offset + 2,
                    raw.purifying_storm.name.get().to_string(),
                    SkillNumbers::of(&purifying_storm),
                    purifying_storm,
                )),
            ]
        }

        BossKind::Goz => vec![
            Box::new(GozMagicalCoinHat::new(
                owner,
                offset,
                name(),
                SkillNumbers {
                    duration: goz_params::MAGICAL_COIN_HAT_DURATION,
                    frames: goz_params::MAGICAL_COIN_HAT_FRAMES,
                    ..SkillNumbers::default()
                },
                (),
            )),
            Box::new(GozNowYouSeeUs::new(
                owner,
                offset + 1,
                name(),
                SkillNumbers::default(),
                (),
            )),
            Box::new(GozThreeLightMonte::new(
                owner,
                offset + 2,
                name(),
                SkillNumbers {
                    cost: goz_params::THREE_LIGHT_MONTE_COST,
                    duration: goz_params::THREE_LIGHT_MONTE_DURATION,
                    frames: goz_params::THREE_LIGHT_MONTE_FRAMES,
                },
                (),
            )),
        ],

        BossKind::Perorodzilla => {
            let params = pero_params::Params::of(difficulty);

            vec![
                Box::new(PerorodzillaWhiteHotHeatVision::new(
                    owner,
                    offset,
                    name(),
                    SkillNumbers {
                        duration: pero_params::DOT_DURATION,
                        frames: pero_params::WHITE_HOT_HEAT_VISION_FRAMES,
                        ..SkillNumbers::default()
                    },
                    params,
                )),
                Box::new(PerorodzillaAquaBall::new(
                    owner,
                    offset + 1,
                    name(),
                    SkillNumbers {
                        frames: pero_params::AQUA_BALL_FRAMES,
                        ..SkillNumbers::default()
                    },
                    params,
                )),
                Box::new(PerorodzillaSummonMinion::new(
                    owner,
                    offset + 2,
                    name(),
                    SkillNumbers {
                        frames: pero_params::SUMMON_MINION_FRAMES,
                        ..SkillNumbers::default()
                    },
                    params,
                )),
                Box::new(PerorodzillaAbsorbMinion::new(
                    owner,
                    offset + 3,
                    name(),
                    SkillNumbers {
                        frames: pero_params::ABSORB_MINION_FRAMES,
                        ..SkillNumbers::default()
                    },
                    params,
                )),
                Box::new(PerorodzillaHyperSpiralGlareBeam::new(
                    owner,
                    offset + 4,
                    name(),
                    SkillNumbers {
                        frames: pero_params::HYPER_SPIRAL_GLARE_BEAM_FRAMES,
                        ..SkillNumbers::default()
                    },
                    params,
                )),
                Box::new(PerorodzillaBurningPerorodzilla::new(
                    owner,
                    offset + 5,
                    name(),
                    SkillNumbers::default(),
                    params,
                )),
            ]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{character::Character, effect::EffectTiming};

    const BINAH: &str = "../../data/bosses/binah.json";

    fn binah(difficulty: Difficulty) -> Boss {
        load(BossKind::Binah, BINAH, ArmorType::Heavy, difficulty, Terrain::Outdoor, 0)
            .expect("failed to load binah")
    }

    #[test]
    fn binah_lunatic_skills() {
        let boss = binah(Difficulty::Lunatic);

        assert_eq!(boss.skills().len(), 3);
        assert_eq!(boss.skills()[2].cost(), 3);
        assert_eq!(boss.skills()[2].duration(), 30);
    }

    #[test]
    fn skill_params_follow_difficulty() {
        let normal = binah(Difficulty::Normal).skills()[0].skill_effects();
        let lunatic = binah(Difficulty::Lunatic).skills()[0].skill_effects();

        assert_ne!(normal[0].targets, lunatic[0].targets);
        assert_eq!(
            normal[1].timing,
            EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 450,
            }
        );
        assert_eq!(
            lunatic[1].timing,
            EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 3600,
            }
        );
    }
}
