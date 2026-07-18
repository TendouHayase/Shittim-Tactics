use core::{agent::Agent, simulator::Simulator, skill::Skill, state::State};
use std::sync::Arc;

use search::algorithm::Algorithm;

pub struct Solver<Sim, Alg, const N: usize> {
    sim: Sim,
    algorithm: Alg,
}

impl<Sim, Alg, const N: usize> Agent for Solver<Sim, Alg, N>
where
    Sim: for<'a> Simulator<S<'a> = State<'a, N>>,
    Alg: for<'a> Algorithm<S<'a> = State<'a, N>>,
{
    type S<'a> = State<'a, N>;

    fn solve<'a>(&self, initial: &Self::S<'a>, threshold: f64) -> Vec<Arc<dyn Skill>> {
        self.algorithm.search(&self.sim, initial.clone(), threshold)
    }
}
