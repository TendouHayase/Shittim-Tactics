#[derive(Debug, Clone, Default)]
pub struct KeiState {
    /// Damage stored by the sub skill.
    pub acc_damage: u64,

    /// Length of the boss damage log when recording started.
    pub recording_start_len: usize,
}
