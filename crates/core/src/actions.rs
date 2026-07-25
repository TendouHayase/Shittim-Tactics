use crate::skill::Skill;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Action {
    pub caster: u32,
    pub targets: Vec<u32>,
    pub skill: Arc<Skill>,
}

#[derive(Debug)]
pub enum ActionContext {
    Wait,
    Use(Action),
}
