pub trait State: Clone {
    type Action: Clone;

    /// Returns whether the skill can be used.
    fn available_action(&self) -> bool;

    /// Returns the actions that can be taken at this time
    fn actions(&self) -> Vec<Self::Action>;

    /// After applying the action, it returns a new state.
    fn apply_actions(&self, action: &Self::Action) -> Self;

    /// Returns whether the battle has ended.
    fn is_terminal() -> bool;

    /// Returns whether the boss has been defeated.
    fn is_goal() -> bool;

    // Returns the cost from the root to the current point.
    fn g_cost() -> u64;
}

pub trait StateHint {
    type HintBundle;

    fn h_inputs(&self) -> Self::HintBundle;
}
