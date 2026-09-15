use core::{actions::ActionContext, agent::Agent, simulator::Simulator, state::State};

/// The default agent for A\*.
///
/// `value` is a lower bound on the frames left before the boss reaches the damage threshold, so
/// it stays admissible; `policy` adds no preference of its own and hands back every legal action.
pub struct Heuristic;

impl Agent for Heuristic {
    type Value = u64;

    fn policy(&self, sim: &impl Simulator, state: &State) -> Vec<(ActionContext, f64)> {
        let actions = sim.legal_actions(state);
        let prior = 1.0 / actions.len() as f64;

        actions.into_iter().map(|action| (action, prior)).collect()
    }

    fn value(&self, _sim: &impl Simulator, state: &State) -> Self::Value {
        state.boss().accumulated_damage().max() // 지금까지 가장 많이 줄어들 수 있는 체력
    }
}
