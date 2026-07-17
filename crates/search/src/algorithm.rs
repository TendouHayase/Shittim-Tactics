use core::{simulator::Simulator, skill::Skill, state::Stateful};
use std::sync::Arc;

pub trait Algorithm {
    type S<'a>: Stateful<'a>;

    fn search<'a>(&self, simulator: &impl Simulator, initial: Self::S<'a>) -> Vec<Arc<dyn Skill>>;
}
