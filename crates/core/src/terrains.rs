use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Terrain {
    #[default]
    Street,
    Outdoor,
    Indoor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerrainCombatPowerState {
    SS,
    S,
    A,
    B,
    C,
    D,
}

impl TerrainCombatPowerState {
    /// 높을수록 큰 값. 선언 순서가 `SS`부터라 `Ord`를 파생하면 뒤집히므로 따로 둠.
    pub fn rank(self) -> u8 {
        match self {
            Self::D => 0,
            Self::C => 1,
            Self::B => 2,
            Self::A => 3,
            Self::S => 4,
            Self::SS => 5,
        }
    }

    /// 한 등급 위. `SS`가 상한이라 더 오르지 않고 그대로 있음.
    pub fn promoted(self) -> Self {
        match self {
            Self::D => Self::C,
            Self::C => Self::B,
            Self::B => Self::A,
            Self::A => Self::S,
            Self::S | Self::SS => Self::SS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainCombatPower {
    street: TerrainCombatPowerState,
    outdoor: TerrainCombatPowerState,
    indoor: TerrainCombatPowerState,
}

impl TerrainCombatPower {
    pub fn new(
        street: TerrainCombatPowerState,
        outdoor: TerrainCombatPowerState,
        indoor: TerrainCombatPowerState,
    ) -> Self {
        TerrainCombatPower {
            street,
            outdoor,
            indoor,
        }
    }

    pub fn get(&self, terrain: Terrain) -> TerrainCombatPowerState {
        match terrain {
            Terrain::Street => self.street,
            Terrain::Outdoor => self.outdoor,
            Terrain::Indoor => self.indoor,
        }
    }

    /// 전용무기 3성 효과. 가장 높은 지형 하나만 오름.
    ///
    /// 현재 모든 학생은 기본 지형적성에 `S`가 정확히 하나뿐이라 어느 것을 올릴지 갈리지
    /// 않고, 결과는 `SS` 하나가 생기는 것임. 그 전제가 깨진 데이터를 조용히 넘기지 않도록
    /// 최고 등급이 둘 이상이면 여기서 걸림.
    pub fn promote_best(&mut self) {
        let best = [Terrain::Street, Terrain::Outdoor, Terrain::Indoor]
            .map(|terrain| (terrain, self.get(terrain).rank()));

        let top = best.iter().map(|(_, rank)| *rank).max().unwrap_or(0);
        let mut tied = best.iter().filter(|(_, rank)| *rank == top);

        let (terrain, _) = tied.next().expect("terrain list is never empty");
        assert!(
            tied.next().is_none(),
            "student has more than one highest terrain adaptation"
        );

        let slot = match terrain {
            Terrain::Street => &mut self.street,
            Terrain::Outdoor => &mut self.outdoor,
            Terrain::Indoor => &mut self.indoor,
        };

        *slot = slot.promoted();
    }

    pub fn get_damage_rate(&self, terrain: Terrain) -> f32 {
        match terrain {
            Terrain::Street => match self.street {
                TerrainCombatPowerState::SS => 1.3,
                TerrainCombatPowerState::S => 1.2,
                TerrainCombatPowerState::A => 1.1,
                TerrainCombatPowerState::B => 1.0,
                TerrainCombatPowerState::C => 0.9,
                TerrainCombatPowerState::D => 0.8,
            },
            Terrain::Outdoor => match self.outdoor {
                TerrainCombatPowerState::SS => 1.3,
                TerrainCombatPowerState::S => 1.2,
                TerrainCombatPowerState::A => 1.1,
                TerrainCombatPowerState::B => 1.0,
                TerrainCombatPowerState::C => 0.9,
                TerrainCombatPowerState::D => 0.8,
            },
            Terrain::Indoor => match self.indoor {
                TerrainCombatPowerState::SS => 1.3,
                TerrainCombatPowerState::S => 1.2,
                TerrainCombatPowerState::A => 1.1,
                TerrainCombatPowerState::B => 1.0,
                TerrainCombatPowerState::C => 0.9,
                TerrainCombatPowerState::D => 0.8,
            },
        }
    }
}
