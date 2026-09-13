use core::{actions::ActionContext, agent::Agent, simulator::Simulator, state::State};

/// The default agent for A\*.
///
/// `value` is a lower bound on the frames left before the boss reaches the damage threshold, so
/// it stays admissible; `policy` adds no preference of its own and hands back every legal action.
pub struct Heuristic;

impl Agent for Heuristic {
    type Value = u64;

    fn policy<'s>(&self, sim: &'s impl Simulator, state: &State) -> Vec<(ActionContext<'s>, f64)> {
        let actions = sim.legal_actions(state);
        let prior = 1.0 / actions.len() as f64;

        actions.into_iter().map(|action| (action, prior)).collect()
    }

    fn value(&self, _sim: &impl Simulator, _state: &State) -> Self::Value {
        // 한 타의 데미지 상한을 계산할 곳이 아직 없어 남은 체력을 프레임으로 바꿀 수 없다.
        // 0은 항상 허용적이라 A*가 균일 비용 탐색으로 동작할 뿐 답은 틀리지 않는다.
        0
    }
}
