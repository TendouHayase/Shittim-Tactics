use core::{base::BaseStats, boss::BossSpec, state::State, terrains::Terrain, types::AttackType};
use std::{collections::HashMap, marker::PhantomData, str::FromStr};

use error::Error;
use serde::{Deserialize, Serialize};

use crate::{boss::Boss, difficulty::Difficulty};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Binah<S> {
    stats: BossSpec,
    difficulty: Difficulty,
    phase_switching_hp: [u64; 2],
    _marker: PhantomData<S>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct DifficultyWrapper {
    #[serde(rename = "BaseStats")]
    stats: BaseStats,
    groggy_gauge: u64,
    groggy_duration: u8,
    phase_switching_hp: [u64; 2],
}

impl<S: State + Sync + Send> Boss for Binah<S> {
    type State = S;
    fn new(
        difficulty: Difficulty,
        attack_type: AttackType,
        terrain: Terrain,
    ) -> Result<Self, Error> {
        let data: HashMap<AttackType, HashMap<Difficulty, DifficultyWrapper>> =
            parsing_json::parse_stats("./data/bosses/binah.json")?;

        let mut result: Binah<S>;

        result.stats.base_stats = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .stats;

        result.stats.name = String::from_str("Binah").expect("str to String converting failed");
        result.stats.terrain = terrain;

        result.stats.groggy_gauge = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .groggy_gauge;
        result.stats.groggy_duration = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .groggy_duration;

        result.difficulty = difficulty;
        result.phase_switching_hp = data
            .get(&attack_type)
            .ok_or(Error::ParsingDataError(
                "can not find attack type key in json".to_string(),
            ))?
            .get(&difficulty)
            .ok_or(Error::ParsingDataError(
                "can not find difficulty key in json".to_string(),
            ))?
            .phase_switching_hp;

        Ok(result)
    }

    fn step(
        &self,
        state: &Self::State,
        action: core::actions::Action<impl core::skill::Skill>,
    ) -> Self::State {
    }
}
