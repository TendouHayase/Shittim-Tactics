use crate::skill::Skill;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Action<T: Skill + ?Sized> {
    pub caster: u32,
    pub targets: Vec<u32>,
    pub skill: Arc<T>,
}

#[derive(Debug)]
pub enum ActionContext<T: Skill + ?Sized> {
    Wait,
    Use(Action<T>),
}
