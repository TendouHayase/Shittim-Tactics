use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Terrain {
    #[default]
    Street,
    Outdoor,
    Indoor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainCombatPowerState {
    SS,
    S,
    A,
    B,
    C,
    D,
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
