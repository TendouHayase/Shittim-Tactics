use core::{agent::Agent, simulator::Simulator, skill::Skill, state::Stateful};
use std::{marker::PhantomData, sync::Arc};

use search::algorithm::Algorithm;

pub struct Solver<'a, Sim, Alg, const N: usize, S: Stateful<'a>> {
    sim: Sim,
    algorithm: Alg,
    _marker: PhantomData<&'a S>,
}

impl<'b, Sim, Alg, const N: usize, S: for<'a> Stateful<'a>> Agent for Solver<'b, Sim, Alg, N, S>
where
    Sim: for<'a> Simulator<S<'a> = S>,
    Alg: for<'a> Algorithm<S<'a> = S>,
{
    type S<'a> = S;

    fn solve<'a>(&self, initial: &Self::S<'a>, threshold: f64) -> Vec<Arc<dyn Skill>> {
        self.algorithm.search(&self.sim, initial.clone(), threshold)
    }
}
