use core::{
    base::BaseStats,
    boss::BossSpec,
    skill::{Effect, Skill},
    state::State,
    terrains::Terrain,
    types::AttackType,
};
use std::{collections::HashMap, marker::PhantomData, rc::Rc, str::FromStr, thread::Builder};

use error::Error;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{boss::Boss, difficulty::Difficulty, state::CommonState};

mod skills;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Binah {
    stats: BossSpec,
    difficulty: Difficulty,
    phase_switching_hp: [u64; 2],
    skills: Vec<Rc<dyn Skill>>,
    effects: Vec<Effect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct DifficultyWrapper {
    #[serde(rename = "BaseStats")]
    stats: BaseStats,
    groggy_gauge: u64,
    groggy_duration: u8,
    phase_switching_hp: [u64; 2],
}

impl Boss for Binah {
    fn from_file(
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
            .ok_or(Error::InvalidData(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::InvalidData(
                "can not find difficulty key in json".to_string(),
            ))?
            .stats;

        // 그로기 게이지
        let groggy_gauge = data
            .get(&attack_type)
            .ok_or(Error::InvalidData(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::InvalidData(
                "can not find difficulty key in json".to_string(),
            ))?
            .groggy_gauge;

        // 그로기 지속시간
        let groggy_duration = data
            .get(&attack_type)
            .ok_or(Error::InvalidData(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::InvalidData(
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
            .ok_or(Error::InvalidData(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::InvalidData(
                "can not find difficulty key in json".to_string(),
            ))?
            .phase_switching_hp;

        // 최종 객체
        let result = Binah::builder()
            .stats(boss_spec)
            .difficulty(difficulty)
            .phase_switching_hp(phase_switching_hp)
            .skills(skills)
            .build();

        Ok(result)
    }

    fn hp(&self) -> u64 {
        self.stats.base_stats.hp
    }

    fn status(&self) -> &Self {
        self
    }
    fn mut_status(&mut self) -> &mut Self {
        self
    }
}
