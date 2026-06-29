use std::hash::Hash;

use crate::{actions::Action, skill::Skill};

pub trait State: Clone + PartialEq + Eq + Hash {
    type S: Skill;

    /// List of valid actions that can be taken
    fn legal_actions(&self) -> Vec<Action<Self::S>>;

    /// Returns the state resulting from applying action a
    fn apply(&self, a: &Action<Self::S>) -> Self;

    /// Returns whether the battle has ended.
    fn is_terminal(&self) -> bool;

    /// Returns whether the boss has been defeated.
    fn is_goal(&self) -> bool;

    // Returns the cost from the root to the current point.
    fn g_cost(&self) -> f32;
}
