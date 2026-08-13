#[derive(Debug, Clone)]
pub struct GozState {}

/// Copied into `StateData::extra` byte for byte, so anything holding a heap pointer (`Vec`,
/// `Box`, `Arc`) would be freed twice. That is why minions are tracked as a counter rather than
/// a list of entities.
///
/// `StateData::new` zeroes `extra` instead of calling `Default`, so an all-zero bit pattern has
/// to mean the start of a fight for every field.
#[derive(Debug, Clone, Default)]
pub struct PerorodzillaState {
    /// Expected damage the minions have taken since the wave was summoned.
    pub minion_damage: u64,

    /// Maximum hp of one big minion. `0` skips knockdown detection.
    pub big_minion_hp: u64,

    /// `boss.accumulated_damage.len()` when the wave was summoned.
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
