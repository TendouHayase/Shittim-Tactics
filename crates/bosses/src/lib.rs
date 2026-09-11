pub mod binah;
pub mod goz;
pub mod macros;
pub mod perorodzilla;
pub mod states;

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
use goz::skills::{GozMagicalCoinHat, GozNowYouSeeUs, GozThreeLightMonte};
use perorodzilla::skills::{
    PerorodzillaAbsorbMinion, PerorodzillaAquaBall, PerorodzillaBurningPerorodzilla,
    PerorodzillaHyperSpiralGlareBeam, PerorodzillaSummonMinion, PerorodzillaWhiteHotHeatVision,
    params::Params,
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

            vec![
                Box::new(BinahAtsilutsLight::new(
                    owner,
                    offset,
                    raw.atsiluts_light.name.get().to_string(),
                    raw.atsiluts_light.pick(difficulty),
                )),
                Box::new(BinahFiresofSeverity::new(
                    owner,
                    offset + 1,
                    raw.fires_of_severity.name.get().to_string(),
                    raw.fires_of_severity.pick(difficulty),
                )),
                Box::new(BinahPurifyingStorm::new(
                    owner,
                    offset + 2,
                    raw.purifying_storm.name.get().to_string(),
                    raw.purifying_storm.pick(difficulty),
                )),
            ]
        }

        BossKind::Goz => vec![
            Box::new(GozMagicalCoinHat::new(owner, offset, name())),
            Box::new(GozNowYouSeeUs::new(owner, offset + 1, name())),
            Box::new(GozThreeLightMonte::new(owner, offset + 2, name())),
        ],

        BossKind::Perorodzilla => {
            let params = Params::of(difficulty);

            vec![
                Box::new(PerorodzillaWhiteHotHeatVision::new(owner, offset, name(), params)),
                Box::new(PerorodzillaAquaBall::new(owner, offset + 1, name(), params)),
                Box::new(PerorodzillaSummonMinion::new(owner, offset + 2, name(), params)),
                Box::new(PerorodzillaAbsorbMinion::new(owner, offset + 3, name(), params)),
                Box::new(PerorodzillaHyperSpiralGlareBeam::new(owner, offset + 4, name(), params)),
                Box::new(PerorodzillaBurningPerorodzilla::new(owner, offset + 5, name(), params)),
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
