use serde::{Deserialize, Serialize};

use crate::{skill::Skill, stat::StatKind, state::StateData};

/// An `EffectKind::Other` body, called with the same `(caster, targets)` routing as
/// [`Skill::apply`].
pub type OtherEffectFn = fn(&dyn Skill, &mut StateData, &mut [&mut StateData]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CCEffect {
    Stun,
    Fear,
    Taunt,
    Confusion,
}

// TODO
pub enum SpecialStatusEffect {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectTiming {
    Instant,
    Persistent {
        interval_frames: u16,
        duration_frames: u16,
    },
}

/// Marks the kind of an applied skill or status effect.
///
/// `Other` compares by function pointer address for `Eq` and `Hash`. Two distinct sources can
/// merge into one address if they compile to the same machine code, which is accepted here:
/// functions that behave identically are identical for this purpose.
#[warn(private_interfaces)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EffectKindOther(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Damage {
        coef_num: u16,
        coef_den: u16,
    },
    Heal {
        coef_num: u16,
        coef_den: u16,
    },
    Buff {
        ty: BuffKind,
        /// Multiplicative, per ten thousand.
        scale: u16,
        /// Additive, in the units of the stat.
        amount: u32,
    },
    Debuff {
        ty: DebuffKind,
        /// Multiplicative, per ten thousand.
        scale: u16,
        /// Additive, in the units of the stat.
        amount: u32,
    },
    Move,
    CC {
        ty: CCEffect,
    },
    Other(EffectKindOther),
}

impl Effect {
    /// The stat this effect changes with its signed `(amount, scale)`, if it changes one at all.
    pub fn stat_change(&self) -> Option<(StatKind, f64, f64)> {
        match *self {
            Effect::Buff { ty, scale, amount } => {
                Some((ty.stat()?, amount as f64, scale as f64 / 10000.0))
            }
            Effect::Debuff { ty, scale, amount } => {
                Some((ty.stat()?, -(amount as f64), -(scale as f64) / 10000.0))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn new_other(func: OtherEffectFn) -> Self {
        Effect::Other(EffectKindOther(func as usize))
    }
    #[inline]
    pub fn is_other(&self) -> bool {
        matches!(self, Effect::Other(_))
    }
    #[inline]
    pub fn as_other(&self) -> Option<OtherEffectFn> {
        match self {
            Effect::Other(ptr) => unsafe {
                if (ptr.0 as *const u8).is_null() {
                    None
                } else {
                    Some(std::mem::transmute::<usize, OtherEffectFn>(ptr.0))
                }
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebuffKind {
    Accuracy,
    Atk,
    AtkSpeed,
    Burn,
    PassionateCheering,
    Chill,
    ChillDmgTaken,
    FocusedAssault,
    CritRes,
    CritDmgRes,
    Crit,
    WeaknessDetection,
    Def,
    Evasion,
    Shock,
    RecoveryBoost,
    MovSpeed,
    CcRes,
    Poison,
    WeaknessDmgTaken,
}

impl DebuffKind {
    /// `None` for kinds that are not a plain stat change, which have no effect yet.
    pub const fn stat(self) -> Option<StatKind> {
        Some(match self {
            DebuffKind::Accuracy => StatKind::Accuracy,
            DebuffKind::Atk => StatKind::Atk,
            DebuffKind::AtkSpeed => StatKind::AtkSpeed,
            DebuffKind::CritRes => StatKind::CritRes,
            DebuffKind::CritDmgRes => StatKind::CritDmgRes,
            DebuffKind::Crit => StatKind::Crit,
            DebuffKind::Def => StatKind::Def,
            DebuffKind::Evasion => StatKind::Evasion,
            DebuffKind::RecoveryBoost => StatKind::RecoveryBoost,
            DebuffKind::MovSpeed => StatKind::MovSpeed,
            DebuffKind::CcRes => StatKind::CcRes,
            DebuffKind::Burn
            | DebuffKind::PassionateCheering
            | DebuffKind::Chill
            | DebuffKind::ChillDmgTaken
            | DebuffKind::FocusedAssault
            | DebuffKind::WeaknessDetection
            | DebuffKind::Shock
            | DebuffKind::Poison
            | DebuffKind::WeaknessDmgTaken => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuffKind {
    Accuracy,
    Atk,
    AtkSpeed,
    ExSkillCost,
    CritRes,
    CritDmg,
    CritDmgRes,
    Crit,
    Def,
    Evasion,
    HpRegen,
    BasicsProficiency,
    ExSkillDmgDealt,
    ExplosiveEffectiveness,
    PiercingEffectiveness,
    MysticEffectiveness,
    SonicEffectiveness,
    RecoveryBoost,
    Healing,
    MaxHp,
    MovSpeed,
    CcPower,
    CcRes,
    CostRecovery,
    Barrier,
    DmgDealt,
    NormalAttackRange,
}

impl BuffKind {
    /// `None` for kinds that are not a plain stat change, which have no effect yet.
    pub const fn stat(self) -> Option<StatKind> {
        Some(match self {
            BuffKind::Accuracy => StatKind::Accuracy,
            BuffKind::Atk => StatKind::Atk,
            BuffKind::AtkSpeed => StatKind::AtkSpeed,
            BuffKind::CritRes => StatKind::CritRes,
            BuffKind::CritDmg => StatKind::CritDmg,
            BuffKind::CritDmgRes => StatKind::CritDmgRes,
            BuffKind::Crit => StatKind::Crit,
            BuffKind::Def => StatKind::Def,
            BuffKind::Evasion => StatKind::Evasion,
            BuffKind::BasicsProficiency => StatKind::BasicsProficiency,
            BuffKind::ExSkillDmgDealt => StatKind::ExSkillDmgDealt,
            BuffKind::ExplosiveEffectiveness => StatKind::ExplosiveEffectiveness,
            BuffKind::PiercingEffectiveness => StatKind::PiercingEffectiveness,
            BuffKind::MysticEffectiveness => StatKind::MysticEffectiveness,
            BuffKind::SonicEffectiveness => StatKind::SonicEffectiveness,
            BuffKind::RecoveryBoost => StatKind::RecoveryBoost,
            BuffKind::Healing => StatKind::Healing,
            BuffKind::MaxHp => StatKind::Hp,
            BuffKind::MovSpeed => StatKind::MovSpeed,
            BuffKind::CcPower => StatKind::CcPower,
            BuffKind::CcRes => StatKind::CcRes,
            BuffKind::CostRecovery => StatKind::CostRecovery,
            BuffKind::DmgDealt => StatKind::DmgDealt,
            BuffKind::NormalAttackRange => StatKind::NormalAttackRange,
            BuffKind::ExSkillCost | BuffKind::HpRegen | BuffKind::Barrier => return None,
        })
    }
}
