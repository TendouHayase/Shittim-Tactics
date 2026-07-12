use std::rc::Rc;

use crate::{
    actions::ActionContext,
    boss::BossBehavior,
    damage::{Damage, key::DamageKey},
    skill::Skill,
    state::{State, Stateful},
};

pub trait Simulator {
    fn legal_actions<'a>(&self, state: &impl Stateful<'a>) -> Vec<Rc<dyn Skill>>;
    fn apply<'a, 'b, 'c>(
        &self,
        state: &'b impl Stateful<'a>,
        action: &'b ActionContext<dyn Skill + 'c>,
    ) -> Result<impl Stateful<'a>, error::Error>;
    fn advance<'a, 'b>(
        &self,
        state: &'b impl Stateful<'a>,
        delta_ticks: u16,
    ) -> Result<impl Stateful<'a>, error::Error>;
    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16;
}
