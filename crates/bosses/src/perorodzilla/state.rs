use core::extra::{ExtraState, ExtraStateData};
use std::{any::TypeId, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct PerorodzillaState {
    /// Expected damage the minions have taken since the wave was summoned.
    pub minion_damage: u64,

    /// Maximum hp of one big minion. `0` skips knockdown detection.
    pub big_minion_hp: u64,

    /// `boss.common.accumulated_damage.len()` when the wave was summoned.
    pub damage_record_start: usize,

    pub big_minions: u8,
    pub shiny_minions: u8,
    pub knocked_down: u8,
    pub small_minions: u8,

    /// The denominator is 7, 10 or 12 depending on difficulty.
    pub groggy_numerator: u8,

    /// Hyper Spiral Glare Beam fires at 100.
    pub atg_percent: u16,
}

impl ExtraState for PerorodzillaState {}
