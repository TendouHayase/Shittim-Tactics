use std::rc::Rc;

use crate::{character::Character, skill::Skill};

#[derive(Debug, Clone)]
pub struct Action<T: Skill + ?Sized> {
    pub caster: u32,
    pub targets: Vec<u32>,
    pub skill: Rc<T>,
}

#[derive(Debug)]
pub enum ActionContext<T: Skill + ?Sized> {
    Wait,
    Use(Action<T>),
}
