use core::{actions::ActionContext, agent::Agent, simulator::Simulator, state::State};

/// The default agent for A\*.
///
/// `value` is a lower bound on the frames left before the boss reaches the damage threshold, so
/// it stays admissible; `policy` adds no preference of its own and hands back every legal action.
pub struct Heuristic;

impl Agent for Heuristic {
    type Value = u64;

    fn policy(&self, sim: &Simulator, state: &State) -> Vec<(ActionContext, f64)> {
        let actions = sim.legal_actions(state);
        let prior = 1.0 / actions.len() as f64;

        actions.into_iter().map(|action| (action, prior)).collect()
    }

    fn value(&self, _sim: &Simulator, _state: &State) -> Self::Value {
        // 최소 시간 하한을 구현하기 전까지 허용성이 자명한 0을 쓴다.
        0
    }
}
