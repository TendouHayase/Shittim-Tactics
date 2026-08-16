use core::{algorithm::Algorithm, skill::Skill};

/// Runs a search and hands back the skill order it found.
///
/// The algorithm is boxed because which one to run is decided at runtime. That is only possible
/// because `Algorithm` mentions no state type — the algorithm holds its own simulator and builds
/// the initial state itself.
pub struct Solver<'a> {
    algorithm: Box<dyn Algorithm<'a> + 'a>,
}

impl<'a> Solver<'a> {
    pub fn new(algorithm: Box<dyn Algorithm<'a> + 'a>) -> Self {
        Solver { algorithm }
    }

    pub fn solve(&self, threshold: f64) -> Vec<(&'a Skill, u16)> {
        self.algorithm.search(threshold)
    }
}
