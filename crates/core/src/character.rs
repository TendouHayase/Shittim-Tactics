use std::{fmt::Debug, sync::Arc};

use crate::{base::BaseStats, boss::Boss, skill::Skill, student::Student};

pub trait CharacterOps {
    fn id(&self) -> u32;
    fn stats(&self) -> &BaseStats;
    fn skill_list(&self) -> &[Skill];
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Character<'a> {
    Student(&'a Student),
    Boss(&'a Boss),
}

impl<'a> CharacterOps for Character<'a> {
    fn id(&self) -> u32 {
        match self {
            Self::Student(s) => s.id(),
            Self::Boss(b) => b.id(),
        }
    }
    fn stats(&self) -> &BaseStats {
        match self {
            Self::Student(s) => s.stats(),
            Self::Boss(b) => b.stats(),
        }
    }
    fn skill_list(&self) -> &[Skill] {
        match self {
            Self::Student(s) => s.skill_list(),
            Self::Boss(b) => b.skill_list(),
        }
    }
}

impl Character<'_> {
    pub fn is_boss(&self) -> bool {
        match self {
            Self::Boss(_) => true,
            _ => false,
        }
    }

    pub fn is_student(&self) -> bool {
        match self {
            Self::Student(_) => true,
            _ => false,
        }
    }
}
