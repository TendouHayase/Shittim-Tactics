use core::{algorithm::Algorithm, skill::Skill};
use error::Error;
use simulator::{Agent, Simulator};

use crate::best_first::{Solution, best_first};

pub struct Astar<'a, A: Agent<Value = u64>> {
    sim: &'a Simulator,
    agent: A,
}

impl<'a, A: Agent<Value = u64>> Astar<'a, A> {
    pub fn new(sim: &'a Simulator, agent: A) -> Self {
        Astar { sim, agent }
    }

    pub fn solve(&self, threshold: f64) -> Result<Solution<'a>, Error> {
        best_first(
            self.sim,
            threshold,
            |state| self.agent.value(self.sim, state),
            // 합법성은 에이전트가 거치는 `Simulator::legal_actions`가 책임진다.
            |state| {
                self.agent
                    .policy(self.sim, state)
                    .into_iter()
                    .map(|(action, _)| action)
                    .collect()
            },
        )
    }
}

impl<'a, A: Agent<Value = u64>> Algorithm<'a> for Astar<'a, A> {
    fn search(&self, threshold: f64) -> Result<Vec<(&'a dyn Skill, u16)>, Error> {
        Ok(self.solve(threshold)?.timeline)
    }
}
