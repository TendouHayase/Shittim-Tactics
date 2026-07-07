use std::collections::LinkedList;
use std::fmt::Debug;
use std::hash::Hash;

use crate::Position;
use crate::base::BaseStats;
use crate::boss::{Boss, BossState, BossTrait};
use crate::damage::{Damage, DamageCache};
use crate::student::{Student, StudentState};
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
    fn frames(&self) -> u32;
    fn skill_type(&self) -> SkillType;
    fn effects(&self) -> Vec<SkillEffect>;
    fn apply(&self, caster: &mut CasterContext, targets: &mut Vec<TargetContext>);
}

#[derive(Debug)]
pub struct CasterContext<'a> {
    pub stats: &'a BaseStats,
    pub effects: &'a mut LinkedList<Effect>,
    pub position: &'a mut Position,
    pub skill_type: SkillType,
}

#[derive(Debug)]
pub struct TargetContext<'a> {
    pub stats: &'a BaseStats,
    pub accumulated_damage: &'a mut Vec<Damage>,
    pub accumulated_damage_cached: &'a mut DamageCache,
    pub effects: &'a mut LinkedList<Effect>,
    pub position: &'a mut Position,
}

#[derive(Debug)]
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
                duration: _,
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

    pub fn from_student(
        student: &'a Student,
        state: &'a mut StudentState,
        skill_type: SkillType,
    ) -> Self {
        CasterContext {
            stats: &student.stats.base_stats,
            effects: &mut state.effects,
            position: &mut state.coordinate,
            skill_type,
        }
    }

    pub fn from_boss<T: BossTrait>(
        boss: &'a Boss<T>,
        state: &'a mut BossState<T>,
        skill_type: SkillType,
    ) -> Self {
        CasterContext {
            stats: &boss.stats.base_stats,
            effects: &mut state.effects,
            position: &mut state.coordinate,
            skill_type,
        }
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
                duration: _,
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

    pub fn from_student(student: &'a Student, state: &'a mut StudentState) -> Self {
        TargetContext {
            stats: &student.stats.base_stats,
            effects: &mut state.effects,
            position: &mut state.coordinate,
            accumulated_damage: &mut state.accumulated_damage,
            accumulated_damage_cached: &mut state.accumulated_damage_cache,
        }
    }

    pub fn from_boss(
        boss: &'a Boss<impl BossTrait>,
        state: &'a mut BossState<impl BossTrait>,
    ) -> Self {
        TargetContext {
            stats: &boss.stats.base_stats,
            accumulated_damage: &mut state.accumulated_damage,
            accumulated_damage_cached: &mut state.accumulated_damage_cache,
            effects: &mut state.effects,
            position: &mut state.coordinate,
        }
    }
}
