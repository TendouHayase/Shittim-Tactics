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

/// `data/bosses/<보스>.json`의 최상위. 방어 타입 키는 보스마다 다르므로 남는 키를 전부
/// 쓸어담는데, 그래서 `skills`처럼 방어 타입이 아닌 최상위 키는 반드시 여기 필드로 선언되어
/// 있어야 함. 빠뜨리면 `ArmorType` 파싱 실패로 나타남.
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
    /// `Box`로 돌려주는 이유는 스킬이 `NonNull<Boss>`로 자기 보스를 가리키기 때문임. 값으로
    /// 돌려주면 반환하면서 보스가 이동해 그 포인터가 전부 끊김.
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

/// 어느 보스의 파일을 읽는지. json의 `id`는 게임이 정한 외부 값이라 코드가 값을 알 수 없고,
/// 그래서 스킬 목록을 고르는 열쇠로 못 씀.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossKind {
    Binah,
    Goz,
    Perorodzilla,
}

/// [`BossKind`]와 Rust 쪽 스킬 목록을 잇는 유일한 지점. 스킬은 코드라서 데이터로 뺄 수 없음.
///
/// 아직 수치 데이터가 없는 보스는 `skills`를 보지 않고 `Params::of(난이도)`로 만듦.
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
    use crate::skill::SkillOps;

    /// 워크스페이스 루트 기준이 아니라 크레이트 루트 기준으로 도는 것에 주의.
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

    /// 난이도별 배열의 색인이 어긋나면 여기서 잡힘.
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
            crate::skill::EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 450,
            }
        );
        assert_eq!(
            lunatic[1].timing,
            crate::skill::EffectTiming::Persistent {
                interval_frames: 90,
                duration_frames: 3600,
            }
        );
    }
}
