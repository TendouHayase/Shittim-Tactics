use std::fmt::Debug;
use std::hash::Hash;

use crate::state::StateData;
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
    Def,
    CostRecovery,
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
    Def,
    CostRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectTiming {
    Instant,
    Persistent {
        interval_frames: u16,
        duration_frames: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Damage,
    Heal,
    Buff {
        ty: Buff,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: Debuff,
        duration: u16,
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
pub struct Effect {
    pub name: &'static str,
    pub kind: EffectKind,
    pub timing: EffectTiming,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillEffectTarget {
    Boss,
    Student,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillEffect {
    pub name: &'static str,
    pub kind: EffectKind,
    pub timing: EffectTiming,
    pub targets: Vec<SkillEffectTarget>,
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
    fn name(&self) -> &str;
    fn owner(&self) -> &str;
    fn cost(&self) -> u8;
    fn frames(&self) -> u16;
    fn skill_type(&self) -> SkillType;
    fn effects(&self) -> Vec<SkillEffect>;
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b Vec<&'c StateData<'a>>,
    ) -> Vec<StateData<'a>>;
}
