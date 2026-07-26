use std::{fmt::Debug, hash::Hash, sync::Arc};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    base::BaseStats, character::Character, difficulty::Difficulty, skill::Skill, terrains::Terrain,
    types::AttackType,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct DifficultyWrapper {
    #[serde(rename = "BaseStats")]
    stats: BaseStats,
    id: u32,
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

impl Boss {
    pub fn id(&self) -> u32 {
        self.stats.id
    }

    pub fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    pub fn skill_list(&self) -> &[Skill] {
        &self.skills
    }

    fn from_file(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
        skill_mask_offset: usize,
    ) -> Result<Self, Error> {
        todo!()
        //     // 기초 스탯
        //     let base_stats = data
        //         .get(&attack_type)
        //         .ok_or(Error::InvalidData(
        //             "can not find attack type key in json".to_string(),
        //         ))?
        //         .get(&difficulty)
        //         .ok_or(Error::InvalidData(
        //             "can not find difficulty key in json".to_string(),
        //         ))?
        //         .stats;

        //     // id
        //     let id = data
        //         .get(&attack_type)
        //         .ok_or(Error::InvalidData(
        //             "can not find attack type key in json".to_string(),
        //         ))?
        //         .get(&difficulty)
        //         .ok_or(Error::InvalidData(
        //             "can not find difficulty key in json".to_string(),
        //         ))?
        //         .id;

        //     // 그로기 게이지
        //     let groggy_gauge = data
        //         .get(&attack_type)
        //         .ok_or(Error::InvalidData(
        //             "can not find attack type key in json".to_string(),
        //         ))?
        //         .get(&difficulty)
        //         .ok_or(Error::InvalidData(
        //             "can not find difficulty key in json".to_string(),
        //         ))?
        //         .groggy_gauge;

        //     // 그로기 지속시간
        //     let groggy_duration = data
        //         .get(&attack_type)
        //         .ok_or(Error::InvalidData(
        //             "can not find attack type key in json".to_string(),
        //         ))?
        //         .get(&difficulty)
        //         .ok_or(Error::InvalidData(
        //             "can not find difficulty key in json".to_string(),
        //         ))?
        //         .groggy_duration;

        //     // 페이즈 전환 체력
        //     let phase_switching_hp = data
        //         .get(&attack_type)
        //         .ok_or(Error::InvalidData(
        //             "can not find attack type key in json".to_string(),
        //         ))?
        //         .get(&difficulty)
        //         .ok_or(Error::InvalidData(
        //             "can not find difficulty key in json".to_string(),
        //         ))?
        //         .phase_switching_hp;

        //     // 보스스펙 빌드
        //     let boss_spec = BossStats::builder()
        //         .name("Binah".to_string())
        //         .base_stats(base_stats)
        //         .terrain(terrain)
        //         .groggy_gauge(groggy_gauge)
        //         .groggy_duration(groggy_duration)
        //         .id(id)
        //         .phase_switching_hp(phase_switching_hp)
        //         .difficulty(difficulty)
        //         .build();

        //     let tmp_skills: Vec<Skill> = vec![];

        //     // 최종 객체
        //     let mut result = Boss::builder().stats(boss_spec).skills(tmp_skills).build();

        //     let skills: Vec<Skill> = vec![
        //         AtsilutsLight::new(&result, skill_mask_offset),
        //         FiresofSeverity1::new(&result, skill_mask_offset + 1),
        //         FireofSeverity2::new(&result, skill_mask_offset + 2),
        //         PurifyingStorm::new(&result, skill_mask_offset + 3),
        //     ];

        //     result.skills = skills;

        //     Ok(result)
    }
}
