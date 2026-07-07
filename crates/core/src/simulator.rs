use std::rc::Rc;

use crate::{actions::ActionContext, boss::BossTrait, skill::Skill, state::State};

pub trait Simulator<T: BossTrait> {
    fn legal_actions(&self, state: &State<T>) -> Vec<Rc<dyn Skill>>;
    fn apply<'a>(&self, state: &'a State<T>, action: &ActionContext<dyn Skill>) -> State<'a, T>;
    fn advance<'a>(&'a self, state: &'a State<T>, delta_ticks: u32) -> State<'a, T>;
    fn next_event_frames(&self, state: &State<T>) -> u32;
}
