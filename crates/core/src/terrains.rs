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
    /// Higher grade, larger value. Written out because the variants start at `SS`, so a derived
    /// `Ord` would run backwards.
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

    /// One grade up. `SS` is the ceiling and stays where it is.
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

    /// The weapon star 3 effect: only the single highest terrain is raised.
    ///
    /// Every student today has exactly one `S` in their base adaptation, so there is no
    /// ambiguity and the result is one `SS`. Data that breaks that assumption is caught here
    /// rather than passed over: two or more at the top grade fails.
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
