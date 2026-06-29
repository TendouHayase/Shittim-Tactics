use core::{base::BaseStats, boss::BossSpec};

use serde::{Deserialize, Serialize};

use crate::{boss::Boss, difficulty::Difficulty};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Binah {
    stats: BossSpec,
    difficulty: Difficulty,
    phase_switching_hp: [u32; 2],
}

impl Boss for Binah {
    fn new(difficulty: &Difficulty) -> Self {
        match difficulty {
            Difficulty::Normal => Self {},
        }
    }
}
