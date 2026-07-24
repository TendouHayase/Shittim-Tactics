use crate::{
    algorithm::Algorithm,
    astar::{heuristics::heuristics, node::Node},
};
use core::{
    actions::ActionContext,
    simulator::Simulator,
    skill::{Skill, SkillEffectTarget},
    state::Stateful,
    utils::Position,
    utils::{euclidean_distance, is_inside},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    marker::PhantomData,
    sync::Arc,
};

pub struct Astar<'a, const N: usize, S: Stateful<'a>> {
    _marker: PhantomData<&'a S>,
}

impl<'b, const N: usize, S: for<'a> Stateful<'a>> Algorithm for Astar<'b, N, S> {
    type S<'a> = S;

    fn search<'a>(
        &self,
        simulator: &impl core::simulator::Simulator<S<'a> = Self::S<'a>>,
        initial: Self::S<'a>,
        threshold: f64,
    ) -> Vec<std::sync::Arc<dyn core::skill::Skill>> {
        // 결과 노드
        let mut result_node = None;

        // open 리스트, close 리스트 생성
        let mut open: BinaryHeap<Reverse<Arc<Node<'b, Self::S<'a>>>>> = BinaryHeap::new();
        let mut closed: HashMap<Self::S<'a>, u64> = HashMap::new();

        // 초기 state의 h 값
        let init_h = heuristics(simulator, &initial);

        // 최초 node
        let init_node = Node::new(initial, 0, init_h);

        open.push(Reverse(Arc::new(init_node)));

        // 탐색 시작
        while let Some(Reverse(node)) = open.pop() {
            // 보스 처치시 종료
            if node.state.is_goal(threshold) {
                result_node = Some(node);
                break;
            };

            // 현재 비용이 과거 기록된 같은 state의 g값보다 크다면 스킵
            if let Some(&best_g) = closed.get(&node.state)
                && best_g <= node.g
            {
                continue;
            }

            // 현재 노드로 g값 갱신
            closed.insert(node.state.clone(), node.g);

            // 다음 이벤트 발생까지 ticks
            let dt = simulator.next_event_frames(&node.state);

            // dt만큼 상황 진행
            if let Ok(advanced) = simulator.advance(&node.state, dt) {
                // 현재 state에서 수행할 수 있는 행동
                let actions = simulator.legal_actions(&advanced);

                // 현재 할 수 있는 작업이 있고 게임이 종료되지 않았으면 실행
                if !actions.is_empty() && !advanced.is_terminated() {
                    // 가능한 행동 모두 수행
                    for action in actions {
                        let caster = action.owner();
                        let caster_id;
                        {
                            caster_id = caster.upgrade().unwrap().id();
                        }

                        // 현재 행동의 스킬 효과
                        let skill_effects = action.skill_effects();
                        let mut targets = vec![];

                        for skill_effect in skill_effects {
                            for target in skill_effect.targets {
                                match target {
                                    // 자신이 타깃이면 자신 추가
                                    SkillEffectTarget::Oneself { kind: _ } => {
                                        targets.push(caster.upgrade().unwrap().id())
                                    }

                                    // 타깃이 학생들이면 타겟팅할 학생 수만큼 캐스터에서 가까운 순으로 추가
                                    SkillEffectTarget::Student { kind: _, count: num } => {
                                        let _caster_arc = caster.upgrade().unwrap();

                                        // 캐스터가 아닌 학생 목록 불러옴
                                        let mut students: Vec<(Position, u32)> = advanced
                                            .students()
                                            .iter()
                                            .map(|student| {
                                                (student.coordinate, student.character.id())
                                            })
                                            .filter(|student| student.1 != caster_id)
                                            .collect();
                                        let mut caster_coord = Default::default();

                                        // 캐스터 좌표 계산
                                        if advanced.boss().character.id() == caster_id {
                                            caster_coord = advanced.boss().coordinate
                                        } else {
                                            for student in advanced.students() {
                                                if student.character.id() == caster_id {
                                                    caster_coord = student.coordinate;
                                                    break;
                                                }
                                            }
                                        }

                                        // 거리순 정렬
                                        students.sort_by(|lhs, rhs| {
                                            euclidean_distance(caster_coord, lhs.0)
                                                .total_cmp(&euclidean_distance(caster_coord, rhs.0))
                                        });

                                        for i in 0..num {
                                            targets.push(students[i as usize].1);
                                        }
                                    }
                                    SkillEffectTarget::Boss { kind: _ } => {
                                        targets.push(advanced.boss().character.id())
                                    }
                                    // 장판기일 경우 범위 내부에 있는 대상 추가
                                    SkillEffectTarget::Land { kind: _, region } => {
                                        let mut caster_coord = Default::default();
                                        if advanced.boss().character.id() == caster_id {
                                            caster_coord = advanced.boss().coordinate
                                        } else {
                                            for student in advanced.students() {
                                                if student.character.id() == caster_id {
                                                    caster_coord = student.coordinate;
                                                    break;
                                                }
                                            }
                                        }

                                        if is_inside(
                                            advanced.boss().coordinate,
                                            region,
                                            caster_coord,
                                        ) {
                                            targets.push(advanced.boss().character.id());
                                        }

                                        for student in advanced.students().iter() {
                                            let student_coord = student.coordinate;

                                            if is_inside(student_coord, region, caster_coord) {
                                                targets.push(student.character.id());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 액션 컨텍스트 생성
                        let action_context: ActionContext<dyn Skill> =
                            ActionContext::Use(core::actions::Action {
                                caster: caster_id,
                                targets,
                                skill: action,
                            });

                        // 스킬 적용
                        let next_state = simulator.apply(&node.state, &action_context);

                        // g, h값 계산
                        let g = next_state.frames().into();
                        let h = heuristics(simulator, &next_state);

                        // 우선순위 큐에 추가
                        open.push(Reverse(Arc::new(Node::from_parent_node(
                            next_state,
                            g,
                            h,
                            node.clone(),
                            action_context,
                        ))));
                    }
                }
            }
        }

        // 역추적으로 스킬 순서 계산
        if let Some(reverse_node) = result_node {
            let mut node = Arc::new(reverse_node);
            let mut result = vec![];
            if let Some(skill) = node.get_action() {
                result.push(skill);
            }

            while let Some(next_node) = node.get_parent() {
                if let Some(skill) = next_node.get_action() {
                    result.push(skill);
                }

                node = next_node.into();
            }

            result.reverse();
            result
        } else {
            vec![]
        }
    }
}
