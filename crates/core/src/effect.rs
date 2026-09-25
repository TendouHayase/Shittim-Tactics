use serde::{Deserialize, Serialize};

use crate::{skill::Skill, state::StateData};

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
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: DebuffKind,
        scale: u16,
        amount: u32,
    },
    Move,
    CC {
        ty: CCEffect,
    },
    Other(EffectKindOther),
}

impl Effect {
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
