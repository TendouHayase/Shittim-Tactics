use crate::astar::node::Node;
use core::{
    agent::Agent, algorithm::Algorithm, simulator::Simulator, skill::Skill, state::Stateful,
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    marker::PhantomData,
    sync::Arc,
};

pub struct Astar<'a, Sim, S: Stateful<'a>, A: Agent<'a, S, Value = u64>> {
    sim: &'a Sim,
    agent: A,
    _marker: PhantomData<fn() -> S>,
}

impl<'a, Sim, S: Stateful<'a>, A: Agent<'a, S, Value = u64>> Astar<'a, Sim, S, A> {
    pub fn new(sim: &'a Sim, agent: A) -> Self {
        Astar {
            sim,
            agent,
            _marker: PhantomData,
        }
    }
}

impl<'a, Sim, S: Stateful<'a>, A: Agent<'a, S, Value = u64>> Algorithm<'a> for Astar<'a, Sim, S, A>
where
    Sim: Simulator<'a, S>,
{
    fn search(&self, threshold: f64) -> Vec<(&'a Skill, u16)> {
        let initial = self.sim.initial_state();

        // 결과 노드
        let mut result_node = None;

        // open 리스트, close 리스트 생성
        let mut open: BinaryHeap<Reverse<Arc<Node<'a, S>>>> = BinaryHeap::new();
        let mut closed: HashMap<S, u64> = HashMap::new();

        // 초기 state의 h 값
        let init_h = self.agent.value(self.sim, &initial);

        // 최초 node
        let init_node = Node::new(initial, 0, init_h);

        open.push(Reverse(Arc::new(init_node)));

        // 탐색 시작
        while let Some(Reverse(node)) = open.pop() {
            // 보스 처치시 종료
            if node.state.is_goal(threshold) {
                result_node = Some(node);
                break;
            }

            // 현재 비용이 과거 기록된 같은 state의 g값보다 크다면 스킵
            if let Some(&best_g) = closed.get(&node.state)
                && best_g <= node.g
            {
                continue;
            }

            // 현재 노드로 g값 갱신
            closed.insert(node.state.clone(), node.g);

            // 다음 이벤트 발생까지 ticks
            let dt = self.sim.next_event_frames(&node.state);

            // dt만큼 상황 진행
            let Ok(advanced) = self.sim.advance(&node.state, dt) else {
                continue;
            };

            if advanced.is_terminated() {
                continue;
            }

            // 무엇을 할 수 있는지는 에이전트에게 묻는다. 합법성은 에이전트가 거치는
            // `Simulator::legal_actions`가 책임지므로 여기서 다시 검사하지 않는다.
            for (action, _) in self.agent.policy(self.sim, &advanced) {
                // 스킬 적용
                let next_state = self.sim.apply(&advanced, &action);

                // g, h값 계산
                let g = next_state.frames().into();
                let h = self.agent.value(self.sim, &next_state);

                // 우선순위 큐에 추가
                open.push(Reverse(Arc::new(Node::from_parent_node(
                    next_state,
                    g,
                    h,
                    node.clone(),
                    action,
                ))));
            }
        }

        // 역추적으로 스킬 순서 계산
        let Some(reverse_node) = result_node else {
            return vec![];
        };

        let mut node = reverse_node;
        let mut result = vec![];

        if let Some(skill) = node.get_action() {
            result.push((skill, node.state.frames()));
        }

        while let Some(next_node) = node.get_parent() {
            if let Some(skill) = next_node.get_action() {
                result.push((skill, next_node.state.frames()));
            }

            node = next_node;
        }

        result.reverse();
        result
    }
}
