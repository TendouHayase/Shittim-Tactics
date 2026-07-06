use std::fmt::Debug;
use std::hash::Hash;

use stochastic::distributions::IrwinHall;

use crate::Position;
use crate::base::BaseStats;
use crate::damage::{Damage, DamageCache};
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

pub struct CasterContext<'a> {
    pub stats: &'a mut BaseStats,
    pub effects: &'a mut Vec<Effect>,
    pub position: &'a mut Position,
    pub skill_type: SkillType,
}

pub struct TargetContext<'a> {
    pub stats: &'a mut BaseStats,
    pub accumulated_damage: &'a mut Vec<Damage>,
    pub accumulated_damage_cached: &'a mut DamageCache,
    pub effects: &'a mut Vec<Effect>,
    pub position: &'a mut Position,
}

pub struct SkillContext<'a> {
    pub name: &'a str,
    pub caster: CasterContext<'a>,
    pub targets: Vec<TargetContext<'a>>,
}

impl<'a> CasterContext<'a> {
    pub fn atk(&self) -> u32 {
        let result = self.stats.atk;
        let mut scale = 100;
        let mut inc = 0;
        for effect in self.effects.iter() {
            if let EffectKind::Buff {
                ty: t,
                duration: d,
                scale: s,
                amount: i,
            } = &effect.kind
            {
                match t {
                    crate::skill::Buff::Atk => {
                        scale *= s;
                        inc += i;
                    }
                    _ => (),
                }
            }
        }

        (result + inc) * (scale as u32)
    }
}

impl<'a> TargetContext<'a> {
    pub fn atk(&self) -> u32 {
        let result = self.stats.atk;
        let mut scale = 100;
        let mut inc = 0;
        for effect in self.effects.iter() {
            if let EffectKind::Buff {
                ty: t,
                duration: d,
                scale: s,
                amount: i,
            } = &effect.kind
            {
                match t {
                    crate::skill::Buff::Atk => {
                        scale *= s;
                        inc += i;
                    }
                    _ => (),
                }
            }
        }

        (result + inc) * (scale as u32)
    }
}
