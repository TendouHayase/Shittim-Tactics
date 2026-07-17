use core::{agent::Agent, simulator::Simulator, skill::Skill, state::State};
use std::{rc::Rc, sync::Arc};

use search::algorithm::Algorithm;

pub struct Solver<Sim, Alg, const N: usize> {
    sim: Sim,
    algorithm: Alg,
}

impl<Sim, Alg, const N: usize> Agent for Solver<Sim, Alg, N>
where
    Sim: Simulator,
    Alg: for<'a> Algorithm<S<'a> = State<'a, N>>,
{
    type S<'a> = State<'a, N>;

    fn solve<'a>(&self, initial: &Self::S<'a>) -> Vec<Arc<dyn Skill>> {
        self.algorithm.search(&self.sim, initial.clone()).into()
    }
}
