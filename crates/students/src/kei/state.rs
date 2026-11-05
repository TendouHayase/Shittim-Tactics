use core::extra::ExtraState;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct KeiExtraState {
    /// Damage stored by the sub skill. Each amplifier that ends overwrites it.
    pub acc_damage: u64,

    /// What the allies in range had dealt when the running amplifier started.
    pub recording_start: u64,
}

impl ExtraState for KeiExtraState {}
