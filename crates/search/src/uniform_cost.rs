use core::{algorithm::Algorithm, skill::Skill};
use error::Error;
use simulator::Simulator;

use crate::best_first::{Solution, best_first};

/// Exhaustive search in order of elapsed frames, with no heuristic.
///
/// Slow but trivially optimal over event-time timelines, which makes it the reference to compare
/// heuristic searches against.
pub struct UniformCost<'a> {
    sim: &'a Simulator,
}

impl<'a> UniformCost<'a> {
    pub fn new(sim: &'a Simulator) -> Self {
        UniformCost { sim }
    }

    pub fn solve(&self, threshold: f64) -> Result<Solution<'a>, Error> {
        best_first(
            self.sim,
            threshold,
            |_| 0,
            |state| self.sim.legal_actions(state),
        )
    }
}

impl<'a> Algorithm<'a> for UniformCost<'a> {
    fn search(&self, threshold: f64) -> Result<Vec<(&'a dyn Skill, u16)>, Error> {
        Ok(self.solve(threshold)?.timeline)
    }
}
