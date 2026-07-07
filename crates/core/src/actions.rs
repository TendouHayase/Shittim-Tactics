use std::rc::Rc;

use crate::{
    character::Character,
    skill::{Skill, SkillContext},
};

#[derive(Debug, Clone)]
pub struct Action<'a, T: Skill + ?Sized> {
    pub caster: &'a dyn Character,
    pub targets: Vec<&'a dyn Character>,
    pub skill: Rc<T>,
}

#[derive(Debug)]
pub enum ActionContext<'a, T: Skill + ?Sized> {
    Wait,
    Use(Action<'a, T>),
}
