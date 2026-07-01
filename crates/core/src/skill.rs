use std::fmt::Debug;
use std::hash::Hash;

use crate::context::SkillContext;
use crate::types::AttackType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Buff {
    Atk,
    Crit,
    CritDmg,
    Effectiveness(AttackType),
    BasicProficiency,
    ExSkillDmgDealt,
    DmgDealt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Debuff {
    Atk,
    Crit,
    CritDmg,
    Effectiveness(AttackType),
    ExSkillDmgDealt,
    BasicProficiency,
    DmgDealt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectTiming {
    Instant,
    Persistent {
        interval_frames: u32,
        duration_frames: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Damage,
    Heal,
    Buff {
        ty: Buff,
        duration: u32,
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: Debuff,
        duration: u32,
        scale: u16,
        amount: u32,
    },
    Move,
    DamageRegion {
        length: u16,
        width: u16,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectTarget {
    Boss,
    Student,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Effect {
    pub name: &'static str,
    pub kind: EffectKind,
    pub timing: EffectTiming,
    pub targets: Vec<EffectTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Ex,
    Basic,
    Enhanced,
    Sub,
    NormalAttack,
}

pub trait Skill: Debug {
    fn cost(&self) -> u8;
    fn frames(&self) -> u32;
    fn effects(&self) -> Vec<Effect>;
    fn apply(&self, ctx: &mut SkillContext);
}
