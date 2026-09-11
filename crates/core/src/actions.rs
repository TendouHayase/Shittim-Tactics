use crate::{skill::Skill, uid::Uid};

#[derive(Debug, Clone)]
pub struct Action<'a> {
    pub caster: Uid,
    pub targets: Vec<Uid>,
    pub skill: &'a dyn Skill,
}

#[derive(Debug)]
pub enum ActionContext<'a> {
    Wait,
    Use(Action<'a>),
}
