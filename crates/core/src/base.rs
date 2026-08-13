use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    stat::StatKind,
    types::{ArmorType, AttackType},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TypedBuilder,
)]
pub struct BaseStats {
    /// Absent from student files, which only carry level 1 stats: the level itself is chosen
    /// when a party is formed, so this is filled in later. Boss files do state it per
    /// difficulty.
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

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub crit_dmg: u32,

    /// Per ten thousand.
    #[builder(default = 5000)]
    pub crit_dmg_res: u32,

    pub stability: u16,

    #[builder(default = 2000)]
    pub stability_rate: u16,

    pub normal_attack_range: u16,

    #[builder(default = 800)]
    pub sighting_range: u16,

    /// Percent.
    #[builder(default = 100)]
    pub cc_power: u16,

    /// Percent.
    #[builder(default = 100)]
    pub cc_res: u16,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub recovery_boost: u32,

    #[builder(default = 700)]
    pub cost_recovery: u16,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub atk_speed: u32,

    #[builder(default = 200)]
    pub mov_speed: u16,

    #[builder(default = 0)]
    pub block_rate_bonus: i16,

    #[builder(default = 0)]
    pub defense_piercing: u16,

    pub mag_count: u8,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub dmg_dealt: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub dmg_resist: u16,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub ex_skill_dmg_dealt: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub ex_skill_dmg_resist: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub basics_proficiency: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub healing_boost: u32,

    pub attack_type: AttackType,
    pub armor_type: ArmorType,

    /// Per ten thousand.
    pub explosive_effectiveness: u32,

    /// Per ten thousand.
    pub piercing_effectiveness: u32,

    /// Per ten thousand.
    pub corrosive_effectiveness: u32,

    /// Per ten thousand.
    pub mystic_effectiveness: u32,

    /// Per ten thousand.
    pub sonic_effectiveness: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub buff_retention: u32,

    /// Per ten thousand.
    #[builder(default = 10000)]
    pub debuff_retention: u32,
}

impl BaseStats {
    /// Writes `(current + amount) * scale` to the matching field.
    ///
    /// Both arguments must already be summed. The game adds every flat increase, then
    /// multiplies by the sum of the rates once, so calling this per term would compound the
    /// rates and round too often. `scale` is a multiplier: no increase is `1.0`.
    pub fn apply_stat(mut self, stat: StatKind, amount: f64, scale: f64) -> Self {
        match stat {
            StatKind::Hp => self.hp = ((self.hp as f64 + amount) * scale).round() as u64,
            StatKind::Atk => self.atk = ((self.atk as f64 + amount) * scale).round() as u32,
            StatKind::Def => self.def = ((self.def as f64 + amount) * scale).round() as u32,
            StatKind::Healing => {
                self.healing = ((self.healing as f64 + amount) * scale).round() as u32
            }
            StatKind::Accuracy => {
                self.accuracy = ((self.accuracy as f64 + amount) * scale).round() as u16
            }
            StatKind::Evasion => {
                self.evasion = ((self.evasion as f64 + amount) * scale).round() as u16
            }
            StatKind::Crit => self.crit = ((self.crit as f64 + amount) * scale).round() as u16,
            StatKind::CritRes => {
                self.crit_res = ((self.crit_res as f64 + amount) * scale).round() as i32
            }
            StatKind::CritDmg => {
                self.crit_dmg = ((self.crit_dmg as f64 + amount) * scale).round() as u32
            }
            StatKind::CritDmgRes => {
                self.crit_dmg_res = ((self.crit_dmg_res as f64 + amount) * scale).round() as u32
            }
            StatKind::Stability => {
                self.stability = ((self.stability as f64 + amount) * scale).round() as u16
            }
            StatKind::StabilityRate => {
                self.stability_rate = ((self.stability_rate as f64 + amount) * scale).round() as u16
            }
            StatKind::NormalAttackRange => {
                self.normal_attack_range =
                    ((self.normal_attack_range as f64 + amount) * scale).round() as u16
            }
            StatKind::SightingRange => {
                self.sighting_range = ((self.sighting_range as f64 + amount) * scale).round() as u16
            }
            StatKind::CcPower => {
                self.cc_power = ((self.cc_power as f64 + amount) * scale).round() as u16
            }
            StatKind::CcRes => self.cc_res = ((self.cc_res as f64 + amount) * scale).round() as u16,
            StatKind::RecoveryBoost => {
                self.recovery_boost = ((self.recovery_boost as f64 + amount) * scale).round() as u32
            }
            StatKind::CostRecovery => {
                self.cost_recovery = ((self.cost_recovery as f64 + amount) * scale).round() as u16
            }
            StatKind::AtkSpeed => {
                self.atk_speed = ((self.atk_speed as f64 + amount) * scale).round() as u32
            }
            StatKind::MovSpeed => {
                self.mov_speed = ((self.mov_speed as f64 + amount) * scale).round() as u16
            }
            StatKind::BlockRateBonus => {
                self.block_rate_bonus =
                    ((self.block_rate_bonus as f64 + amount) * scale).round() as i16
            }
            StatKind::DefensePiercing => {
                self.defense_piercing =
                    ((self.defense_piercing as f64 + amount) * scale).round() as u16
            }
            StatKind::MagCount => {
                self.mag_count = ((self.mag_count as f64 + amount) * scale).round() as u8
            }
            StatKind::DmgDealt => {
                self.dmg_dealt = ((self.dmg_dealt as f64 + amount) * scale).round() as u32
            }
            StatKind::DmgResist => {
                self.dmg_resist = ((self.dmg_resist as f64 + amount) * scale).round() as u16
            }
            StatKind::ExSkillDmgDealt => {
                self.ex_skill_dmg_dealt =
                    ((self.ex_skill_dmg_dealt as f64 + amount) * scale).round() as u32
            }
            StatKind::ExSkillDmgResist => {
                self.ex_skill_dmg_resist =
                    ((self.ex_skill_dmg_resist as f64 + amount) * scale).round() as u32
            }
            StatKind::BasicsProficiency => {
                self.basics_proficiency =
                    ((self.basics_proficiency as f64 + amount) * scale).round() as u32
            }
            StatKind::HealingBoost => {
                self.healing_boost = ((self.healing_boost as f64 + amount) * scale).round() as u32
            }
            StatKind::ExplosiveEffectiveness => {
                self.explosive_effectiveness =
                    ((self.explosive_effectiveness as f64 + amount) * scale).round() as u32
            }
            StatKind::PiercingEffectiveness => {
                self.piercing_effectiveness =
                    ((self.piercing_effectiveness as f64 + amount) * scale).round() as u32
            }
            StatKind::CorrosiveEffectiveness => {
                self.corrosive_effectiveness =
                    ((self.corrosive_effectiveness as f64 + amount) * scale).round() as u32
            }
            StatKind::MysticEffectiveness => {
                self.mystic_effectiveness =
                    ((self.mystic_effectiveness as f64 + amount) * scale).round() as u32
            }
            StatKind::SonicEffectiveness => {
                self.sonic_effectiveness =
                    ((self.sonic_effectiveness as f64 + amount) * scale).round() as u32
            }
            StatKind::BuffRetention => {
                self.buff_retention = ((self.buff_retention as f64 + amount) * scale).round() as u32
            }
            StatKind::DebuffRetention => {
                self.debuff_retention =
                    ((self.debuff_retention as f64 + amount) * scale).round() as u32
            }
        }

        self
    }
}
