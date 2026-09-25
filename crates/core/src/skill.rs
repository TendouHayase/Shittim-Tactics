use crate::character::Character;
use crate::effect::{Effect, EffectTiming};
use crate::simulator::Simulator;
use crate::state::State;
use crate::uid::{SkillUid, Uid};
use crate::utils::Position;
use error::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Weak;

#[derive(Debug)]
pub struct SkillHeader {
    pub owner: Uid,
    pub owner_offset: usize,
    pub name: String,
    pub skill_offset: usize,
    pub skill_type: SkillType,
    pub effects: Vec<SkillEffect>,
    pub cost: u8,
    pub duration: u16, // 시전후 시전 종료까지의 시간
    pub frames: u16,   // 발동후 시전까지 시간
    sim: Weak<Simulator>,
}

impl SkillHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner: Uid,
        owner_offset: usize,
        name: &str,
        skill_offset: usize,
        skill_type: SkillType,
        effects: Vec<SkillEffect>,
        cost: u8,
        duration: u16,
        frames: u16,
        sim: Weak<Simulator>,
    ) -> Self {
        Self {
            owner,
            owner_offset,
            name: name.to_string(),
            skill_offset,
            skill_type,
            effects,
            cost,
            duration,
            frames,
            sim,
        }
    }
}

pub trait Skill: SkillMeta + Debug + Send + Sync {
    fn apply(&self, state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State;
}

pub trait SkillMeta {
    fn header(&self) -> &SkillHeader;

    fn name(&self) -> &str {
        &self.header().name
    }
    fn uid(&self) -> SkillUid {
        SkillUid::new(self.owner_uid(), self.skill_offset())
    }
    fn owner_uid(&self) -> Uid {
        self.header().owner
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
    fn skill_effects(&self) -> &[SkillEffect] {
        &self.header().effects
    }
    fn boss_uid(&self) -> Result<Uid, Error> {
        Ok(self
            .header()
            .sim
            .upgrade()
            .ok_or(Error::ExpiredRefError("referenced value has been dropped"))?
            .boss
            .uid())
    }
    fn other_students_uid(&self) -> Result<Vec<Uid>, Error> {
        let mut out = Vec::with_capacity(8);
        self.header()
            .sim
            .upgrade()
            .ok_or(Error::ExpiredRefError("referenced value has been dropped"))?
            .students
            .iter()
            .for_each(|student| {
                if student.uid() != self.owner_uid() {
                    out.push(student.uid());
                }
            });
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillEffectTarget {
    Boss { kind: Effect },
    Student { kind: Effect, count: u8 },
    Land { kind: Effect, region: Region },
    Oneself { kind: Effect },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillEffect {
    pub id: (Uid, usize),
    pub timing: EffectTiming,
    pub targets: SkillEffectTarget,
}

impl SkillEffect {
    pub fn new(id: (Uid, usize), timing: EffectTiming, targets: SkillEffectTarget) -> Self {
        Self {
            id,
            timing,
            targets,
        }
    }
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
