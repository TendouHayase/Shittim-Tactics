use std::{rc::Rc, sync::Arc};

use crate::{actions::ActionContext, skill::Skill, state::Stateful};

pub trait Agent {
    type S<'a>: Stateful<'a>;

    fn solve<'a>(&self, initial: &Self::S<'a>) -> Vec<Arc<dyn Skill>>;
}

pub trait RLAgent: Agent {
    /// Returns the prior probability/score for each action in state s
    fn policy<'a>(&self, s: &Self::S<'a>) -> Vec<(ActionContext<dyn Skill>, f64)>;

    /// Estimating the Value of State s
    fn value<'a>(&self, s: &Self::S<'a>) -> f64;
}
