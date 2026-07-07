use std::rc::Rc;

use crate::{actions::ActionContext, boss::BossTrait, skill::Skill, state::State};

pub trait Simulator<T: BossTrait> {
    fn legal_actions(&self, state: &State<T>) -> Vec<Rc<dyn Skill>>;
    fn apply(&self, state: &State<T>, action: &ActionContext<dyn Skill>) -> State<T>;
}
