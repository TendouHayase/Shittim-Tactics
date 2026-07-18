use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Weak;

use crate::Position;
use crate::character::Character;
use crate::state::StateData;
use crate::types::AttackType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuffType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebuffType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectTiming {
    Instant,
    Persistent {
        interval_frames: u16,
        duration_frames: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Damage,
    Heal,
    Buff {
        ty: BuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: DebuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Move,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Effect {
    pub name: &'static str,
    pub kind: EffectKind,
    pub timing: EffectTiming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillEffectTarget {
    Boss { kind: EffectKind },
    Student { kind: EffectKind, count: u8 },
    Land { kind: EffectKind, region: Region },
    Oneself { kind: EffectKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Polygon {
        // 왼쪽 위부터 시계방향
        vertex: [Position; 4],
        count: u8,
    },
    Arc {
        radius: u16,
        start_angle_degree: u16,
        end_angle_degree: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillEffect {
    pub name: &'static str,
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

pub trait Skill: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn owner(&self) -> Weak<dyn Character>;
    fn cost(&self) -> u8;
    fn frames(&self) -> u16;
    fn duration(&self) -> u16;
    fn skill_mask_index(&self) -> usize;
    fn skill_type(&self) -> SkillType;
    fn skill_effects(&self) -> Vec<SkillEffect>;
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>>;
}
