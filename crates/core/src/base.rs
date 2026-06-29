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

    #[builder(default = 2000)]
    pub stability_rate: u16,

    pub normal_attack_range: u16,

    #[builder(default = 800)]
    pub sighting_range: u16,

    #[builder(default = 100)]
    pub cc_power: u8,

    #[builder(default = 100)]
    pub cc_res: u8,

    #[builder(default = 10000)]
    pub recovery_boost: u32,

    #[builder(default = 700)]
    pub cost_recovery: u16,

    #[builder(default = 10000)]
    pub atk_speed: u32,

    #[builder(default = 200)]
    pub mov_speed: u16,

    #[builder(default = 0)]
    pub block_rate_bonus: i16,

    #[builder(default = 0)]
    pub defense_piercing: u16,

    pub mag_count: u8,

    #[builder(default = 10000)]
    pub dmg_dealt: u32,

    #[builder(default = 10000)]
    pub dmg_resist: u16,

    #[builder(default = 10000)]
    pub ex_skill_dmg_dealt: u32,

    #[builder(default = 10000)]
    pub ex_skill_dmg_resist: u32,

    #[builder(default = 10000)]
    pub basics_proficiency: u32,

    #[builder(default = 10000)]
    pub healing_boost: u32,

    pub attack_type: AttackType,
    pub armor_type: ArmorType,

    pub explosive_effectiveness: u32,
    pub piercing_effectiveness: u32,
    pub corrosive_effectiveness: u32,
    pub mystic_effectiveness: u32,
    pub sonic_effectiveness: u32,

    #[builder(default = 10000)]
    pub buff_retention: u32,

    #[builder(default = 10000)]
    pub debuff_retention: u32,
}
