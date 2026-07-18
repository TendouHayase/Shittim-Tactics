use crate::{
    algorithm::Algorithm,
    astar::{heuristics::heuristics, node::Node},
};
use core::{
    Position,
    actions::ActionContext,
    simulator::Simulator,
    skill::{Region, Skill, SkillEffectTarget},
    state::{State, Stateful},
    utils::{cross_product, euclidean_distance, is_inside},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

pub struct Astar<const N: usize> {}

impl<const N: usize> Algorithm for Astar<N> {
    type S<'a> = State<'a, N>;

    fn search<'a>(
        &self,
        simulator: &impl core::simulator::Simulator<S<'a> = Self::S<'a>>,
        initial: Self::S<'a>,
        threshold: f64,
    ) -> Vec<std::sync::Arc<dyn core::skill::Skill>> {
        let mut result_node = None;

        let mut open: BinaryHeap<Reverse<Arc<Node<'a, Self::S<'a>>>>> = BinaryHeap::new();
        let mut closed: HashMap<Self::S<'a>, u64> = HashMap::new();

        let init_h = heuristics(simulator, &initial);

        let init_node = Node::new(initial, 0, init_h);

        open.push(Reverse(Arc::new(init_node)));

        while let Some(Reverse(node)) = open.pop() {
            if node.state.is_goal(threshold) {
                result_node = Some(node);
                break;
            };

            if let Some(&best_g) = closed.get(&node.state)
                && best_g <= node.g
            {
                continue;
            }
            closed.insert(node.state.clone(), node.g);

            let dt = simulator.next_event_frames(&node.state);
            if let Ok(advanced) = simulator.advance(&node.state, dt) {
                let actions = simulator.legal_actions(&advanced);

                if !actions.is_empty() || advanced.is_terminated() {
                    for action in actions {
                        let caster = action.owner();
                        let caster_id;
                        {
                            caster_id = caster.upgrade().unwrap().id();
                        }
                        let skill_effects = action.skill_effects();
                        let mut targets = vec![];

                        for skill_effect in skill_effects {
                            for target in skill_effect.targets {
                                match target {
                                    SkillEffectTarget::Oneself { kind } => {
                                        targets.push(caster.upgrade().unwrap().id())
                                    }
                                    SkillEffectTarget::Student { kind, count: num } => {
                                        let _caster_arc = caster.upgrade().unwrap();

                                        let mut students: Vec<(Position, u32)> = advanced
                                            .students()
                                            .iter()
                                            .map(|student| {
                                                (student.coordinate, student.character.id())
                                            })
                                            .filter(|student| student.1 != caster_id)
                                            .collect();
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

                                        students.sort_by(|lhs, rhs| {
                                            euclidean_distance(caster_coord, lhs.0)
                                                .total_cmp(&euclidean_distance(caster_coord, rhs.0))
                                        });

                                        for i in 0..num {
                                            targets.push(students[i as usize].1);
                                        }
                                    }
                                    SkillEffectTarget::Boss { kind } => {
                                        targets.push(advanced.boss().character.id())
                                    }
                                    SkillEffectTarget::Land { kind, region } => {
                                        let mut caster_coord = Default::default();
                                        if advanced.boss.character.id() == caster_id {
                                            caster_coord = advanced.boss.coordinate
                                        } else {
                                            for student in advanced.students() {
                                                if student.character.id() == caster_id {
                                                    caster_coord = student.coordinate;
                                                    break;
                                                }
                                            }
                                        }

                                        if is_inside(advanced.boss.coordinate, region, caster_coord)
                                        {
                                            targets.push(advanced.boss.character.id());
                                        }

                                        for student in advanced.students.iter() {
                                            let student_coord = student.coordinate;

                                            if is_inside(student_coord, region, caster_coord) {
                                                targets.push(student.character.id());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let action_context: ActionContext<dyn Skill> =
                            ActionContext::Use(core::actions::Action {
                                caster: caster_id,
                                targets,
                                skill: action,
                            });
                        let next_state = simulator.apply(&node.state, &action_context);
                        let g = next_state.frames().into();
                        let h = heuristics(simulator, &next_state);
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

            result
        } else {
            vec![]
        }
    }
}
