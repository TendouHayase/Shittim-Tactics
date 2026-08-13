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

impl BaseStats {
    /// `(현재값 + amount) * scale`을 해당 필드에 씀.
    ///
    /// `amount`와 `scale`은 각각 합산이 끝난 값이어야 함. 게임이 증가량을 전부 더한 뒤 증가율의
    /// 합을 한 번 곱하는 방식이라, 항마다 부르면 반올림과 복리가 둘 다 어긋남. `scale`은 배수라
    /// 증가 없음이 `1.0`임.
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
