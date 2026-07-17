use core::{simulator::Simulator, skill::Skill, state::Stateful};
use std::sync::Arc;

pub trait Algorithm {
    type S<'a>: Stateful<'a>;

    fn search<'a>(
        &self,
        simulator: &impl Simulator<S<'a> = Self::S<'a>>,
        initial: Self::S<'a>,
        threshold: f64,
    ) -> Vec<Arc<dyn Skill>>;
}
