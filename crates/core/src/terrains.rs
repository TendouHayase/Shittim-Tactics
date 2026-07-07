use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Street,
    Outdoor,
    Indoor,
}

impl Default for Terrain {
    fn default() -> Self {
        Self::Street
    }
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
                TerrainCombatPowerState::SS => return 1.3,
                TerrainCombatPowerState::S => return 1.2,
                TerrainCombatPowerState::A => return 1.1,
                TerrainCombatPowerState::B => return 1.0,
                TerrainCombatPowerState::C => return 0.9,
                TerrainCombatPowerState::D => return 0.8,
            },
            Terrain::Outdoor => match self.outdoor {
                TerrainCombatPowerState::SS => return 1.3,
                TerrainCombatPowerState::S => return 1.2,
                TerrainCombatPowerState::A => return 1.1,
                TerrainCombatPowerState::B => return 1.0,
                TerrainCombatPowerState::C => return 0.9,
                TerrainCombatPowerState::D => return 0.8,
            },
            Terrain::Indoor => match self.indoor {
                TerrainCombatPowerState::SS => return 1.3,
                TerrainCombatPowerState::S => return 1.2,
                TerrainCombatPowerState::A => return 1.1,
                TerrainCombatPowerState::B => return 1.0,
                TerrainCombatPowerState::C => return 0.9,
                TerrainCombatPowerState::D => return 0.8,
            },
        }
    }
}
