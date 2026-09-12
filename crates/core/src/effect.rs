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
struct EffectKindOther(*const u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Damage {
        coef_num: u16,
        coef_den: u16,
    },
    Heal {
        coef_num: u16,
        coef_den: u16,
    },
    Buff {
        ty: StatKind,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: StatKind,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Move,
    CC {
        ty: CCEffect,
        duration: u16,
    },
    Other(EffectKindOther),
}

impl EffectKind {
    #[inline]
    pub fn new_other(func: OtherEffectFn) -> Self {
        EffectKind::Other(EffectKindOther(func as *const u8))
    }
    #[inline]
    pub fn is_other(&self) -> bool {
        if let EffectKind::Other(_) = self {
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn as_other(&self) -> Option<OtherEffectFn> {
        match self {
            EffectKind::Other(ptr) => unsafe {
                if ptr.0.is_null() {
                    None
                } else {
                    Some(std::mem::transmute::<*const u8, OtherEffectFn>(ptr.0))
                }
            },
            _ => None,
        }
    }
}
