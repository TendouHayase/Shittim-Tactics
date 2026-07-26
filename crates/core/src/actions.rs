use crate::skill::Skill;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Action<'a> {
    pub caster: u32,
    pub targets: Vec<u32>,
    pub skill: &'a Skill,
}

#[derive(Debug)]
pub enum ActionContext<'a> {
    Wait,
    Use(Action<'a>),
}
