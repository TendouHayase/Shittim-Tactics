use crate::effect::{CCEffect, EffectKind, EffectTiming};
use crate::state::StateData;
use crate::uid::Uid;
use crate::utils::Position;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct SkillHeader {
    pub owner: Uid,
    pub owner_offset: usize,
    pub name: String,
    pub skill_offset: usize,
    pub skill_type: SkillType,
    pub cost: u8,
    pub duration: u16,
    pub frames: u16,
}

pub trait Skill: SkillMeta + Debug + Send + Sync {
    fn skill_effects(&self) -> Vec<SkillEffect>;
    fn apply<'b, 'c: 'b>(&self, caster: &'c mut StateData, targets: &'b mut [&'c mut StateData]);
}

pub trait SkillMeta {
    fn header(&self) -> &SkillHeader;

    fn name(&self) -> &str {
        &self.header().name
    }
    fn owner(&self) -> Uid {
        self.header().owner
    }
    fn id(&self) -> (Uid, usize) {
        (self.header().owner, self.header().owner_offset)
    }
    fn cost(&self) -> u8 {
        self.header().cost
    }
    fn duration(&self) -> u16 {
        self.header().duration
    }
    fn frames(&self) -> u16 {
        self.header().frames
    }
    fn skill_offset(&self) -> usize {
        self.header().skill_offset
    }
    fn skill_type(&self) -> SkillType {
        self.header().skill_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillEffectTarget {
    Boss { kind: EffectKind },
    Student { kind: EffectKind, count: u8 },
    Land { kind: EffectKind, region: Region },
    Oneself { kind: EffectKind },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillEffect {
    pub id: (Uid, u8),
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
    /// A boss passive, applied at the start of a fight and held for its duration.
    Passive,
}

/// Numeric parameters of a skill.
///
/// All three have defaults because `#[skill]` only sees the name of the params type, never its
/// fields, and so cannot tell whether a given value exists.
pub trait SkillParams {
    fn cost(&self) -> u8 {
        0
    }
    fn duration(&self) -> u16 {
        0
    }
    fn frames(&self) -> u16 {
        0
    }
}

/// For skills that carry no numbers.
impl SkillParams for () {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum Region {
    Polygon {
        /// Always four entries; `count` says how many are real and the rest are ignored.
        vertex: [Position; 4],
        count: u8,
    },
    Arc {
        radius: u16,
        start_angle_degree: u16,
        end_angle_degree: u16,
    },
}
