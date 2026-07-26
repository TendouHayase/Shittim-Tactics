use std::{fmt::Debug, sync::Arc};

use crate::{base::BaseStats, boss::Boss, skill::Skill, student::Student};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Character<'a> {
    Student(&'a Student),
    Boss(&'a Boss),
}

impl<'a> Character<'a> {
    pub fn id(&self) -> u32 {
        match self {
            Self::Student(s) => s.id(),
            Self::Boss(b) => b.id(),
        }
    }
    pub fn stats(&self) -> &BaseStats {
        match self {
            Self::Student(s) => s.stats(),
            Self::Boss(b) => b.stats(),
        }
    }
    pub fn skill_list(&self) -> &[Skill] {
        match self {
            Self::Student(s) => s.skill_list(),
            Self::Boss(b) => b.skill_list(),
        }
    }
}
