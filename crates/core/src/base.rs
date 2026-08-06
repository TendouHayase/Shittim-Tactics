use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::{ArmorType, AttackType};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TypedBuilder,
)]
pub struct BaseStats {
    /// 학생 파일에는 없음. 레벨은 데이터가 아니라 편성에서 정해지는 런타임 값이라, 파일에서
    /// 읽는 것은 1레벨 스탯뿐이고 여기는 나중에 채워짐. 보스 파일에는 난이도마다 적혀 있음.
    #[serde(default)]
    #[builder(default = 0)]
    pub level: u8,

    pub hp: u64,

    pub atk: u32,

    pub def: u32,

    pub healing: u32,

    pub accuracy: u16,

    pub evasion: u16,

    #[builder(default = 10000)]
    pub crit: u16,

    #[builder(default = 5000)]
    pub crit_res: i32,

    /// 만분율
    #[builder(default = 10000)]
    pub crit_dmg: u32,

    // 만분율
    #[builder(default = 5000)]
    pub crit_dmg_res: u32,

    pub stability: u16,

    #[builder(default = 2000)]
    pub stability_rate: u16,

    pub normal_attack_range: u16,

    #[builder(default = 800)]
    pub sighting_range: u16,

    /// 백분율 단위
    #[builder(default = 100)]
    pub cc_power: u16,

    /// 백분율 단위
    #[builder(default = 100)]
    pub cc_res: u16,

    /// 만분율
    #[builder(default = 10000)]
    pub recovery_boost: u32,

    #[builder(default = 700)]
    pub cost_recovery: u16,

    /// 만분율
    #[builder(default = 10000)]
    pub atk_speed: u32,

    #[builder(default = 200)]
    pub mov_speed: u16,

    #[builder(default = 0)]
    pub block_rate_bonus: i16,

    #[builder(default = 0)]
    pub defense_piercing: u16,

    pub mag_count: u8,

    /// 만분율
    #[builder(default = 10000)]
    pub dmg_dealt: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub dmg_resist: u16,

    /// 만분율
    #[builder(default = 10000)]
    pub ex_skill_dmg_dealt: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub ex_skill_dmg_resist: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub basics_proficiency: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub healing_boost: u32,

    pub attack_type: AttackType,
    pub armor_type: ArmorType,

    /// 만분율
    pub explosive_effectiveness: u32,

    /// 만분율
    pub piercing_effectiveness: u32,

    /// 만분율
    pub corrosive_effectiveness: u32,

    /// 만분율
    pub mystic_effectiveness: u32,

    /// 만분율
    pub sonic_effectiveness: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub buff_retention: u32,

    /// 만분율
    #[builder(default = 10000)]
    pub debuff_retention: u32,
}
