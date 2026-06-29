use core::{
    base::BaseStats, boss::BossSpec, skill::Skill, state::State, terrains::Terrain,
    types::AttackType,
};
use std::{collections::HashMap, marker::PhantomData, str::FromStr, thread::Builder};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{boss::Boss, difficulty::Difficulty};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct Binah<St, Sk> {
    stats: BossSpec,
    difficulty: Difficulty,
    phase_switching_hp: [u64; 2],

    #[builder(default)]
    _marker1: PhantomData<St>,
    #[builder(default)]
    _marker2: PhantomData<Sk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct DifficultyWrapper {
    #[serde(rename = "BaseStats")]
    stats: BaseStats,
    groggy_gauge: u64,
    groggy_duration: u8,
    phase_switching_hp: [u64; 2],
}

impl<St: State + Sync + Send, Sk: Skill + Send + Sync> Boss for Binah<St, Sk> {
    type State = St;
    type Skill = Sk;
    fn new(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error> {
        // json 데이터 가져오기
        let data: HashMap<AttackType, HashMap<Difficulty, DifficultyWrapper>> =
            parsing_json::read_json("./data/bosses/binah.json")?;

        // 기초 스탯
        let base_stats = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .stats;

        // 그로기 게이지
        let groggy_gauge = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .groggy_gauge;

        // 그로기 지속시간
        let groggy_duration = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .groggy_duration;

        // 보스스펙 빌드
        let boss_spec = BossSpec::builder()
            .name("Binah".to_string())
            .base_stats(base_stats)
            .terrain(terrain)
            .groggy_gauge(groggy_gauge)
            .groggy_duration(groggy_duration)
            .build();

        // 페이즈 전환 체력
        let phase_switching_hp = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .phase_switching_hp;

        // 최종 객체
        let result = Binah::builder()
            .stats(boss_spec)
            .difficulty(difficulty)
            .phase_switching_hp(phase_switching_hp)
            .build();

        Ok(result)
    }

    fn step(&self, state: &Self::State, action: core::actions::Action<Self::Skill>) -> Self::State {
        todo!()
    }
}
