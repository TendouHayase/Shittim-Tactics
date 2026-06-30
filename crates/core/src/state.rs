use std::hash::Hash;

use crate::{actions::Action, skill::Skill};

pub trait State: Clone + PartialEq + Eq + Hash {
    /// Returns its own status.
    fn status(&self) -> &Self;

    /// Returns a mutable representation of its own state.
    fn mut_status(&mut self) -> &mut Self;

    /// Returns whether the battle has ended.
    fn is_terminal(&self) -> bool;

    /// Returns whether the boss has been defeated.
    fn is_goal(&self) -> bool;

    // Returns the cost from the root to the current point.
    fn g_cost(&self) -> f32;
}
