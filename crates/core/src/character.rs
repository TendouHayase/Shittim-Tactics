use std::{fmt::Debug, rc::Rc};

use crate::{
    Position,
    base::BaseStats,
    skill::{SkillEffect, Skill},
};

pub trait Character: Debug {
    fn id(&self) -> u32;
    fn stats(&self) -> &BaseStats;
    fn skill_list(&self) -> Rc<Vec<Rc<dyn Skill>>>;
}
