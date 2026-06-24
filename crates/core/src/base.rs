use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::{ArmorType, AttackType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct BaseStats {
    pub level: u8,
    pub hp: u64,
    pub atk: u32,
    pub def: u32,
    pub healing: u32,
    pub accuracy: u16,
    pub evasion: u16,

    #[builder(default = 10000)]
    pub crit: u16,

    #[builder(default = 10000)]
    pub crit_dmg: u32,

    #[builder(default = 5000)]
    pub crit_res: i32,

    pub stability: u16,
    pub normal_attack_rate: u16,

    #[builder(default = 800)]
    pub sighting_range: u16,

    #[builder(default = 0)]
    pub defense_piercing: u16,

    #[builder(default = 10000)]
    pub dmg_resist: u16,

    #[builder(default = 100)]
    pub cc_power: u8,

    #[builder(default = 100)]
    pub cc_res: u8,

    pub attack_type: AttackType,
    pub armor_type: ArmorType,

    pub explosive_effectiveness: u32,
    pub piercing_effectiveness: u32,
    pub corrosive_effectiveness: u32,
    pub mystic_effectiveness: u32,
    pub sonic_effectiveness: u32,
}
