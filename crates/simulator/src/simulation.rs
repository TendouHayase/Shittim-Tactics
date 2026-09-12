use core::{
    actions::{
        Action,
        ActionContext::{self},
    },
    boss::Boss,
    character::Character,
    constants::TPS,
    damage::{Damage, key::SkillsBitMask},
    simulator::Simulator,
    skill::{Skill, SkillEffectTarget::Land, SkillMeta, SkillOps},
    state::{AccumulatedDamage, CommonStateData, RemainedEffects, State, StateData, Stateful},
    student::Student,
    utils::is_inside,
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

pub struct Simulation {
    pub students: Vec<Box<Student>>,
    pub boss: Box<Boss>,

    limit_ticks: u16,

    cost_charge_time: HashMap<SkillsBitMask, u16>,
}

impl Simulator for Simulation {
    fn initial_state(&self) -> State {
        let mut it = self.students.iter();

        if self.students.len() == 6 {
            State {
                students: core::state::StudentState::TotalAssault(std::array::from_fn(|i| {
                    StateData::new(self.students[i].uid())
                })),
                boss: StateData::new(self.boss.uid()),
                frames: 0,
                cost: 0,
            }
        } else if self.students.len() == 10 {
            State {
                students: core::state::StudentState::FinalRestrictionRelease(std::array::from_fn(
                    |i| StateData::new(self.students[i].uid()),
                )),
                boss: StateData::new(self.boss.uid()),
                frames: 0,
                cost: 0,
            }
        } else {
            panic!("unsupported students party size: {}", self.students.len())
        }
    }

    fn legal_actions(&self, state: &State) -> Vec<ActionContext<'_>> {
        let cost = state.cost();
        let mut result = vec![];
        for (i, stat) in state.students().iter().enumerate() {
            for (j, cooltime) in stat.cooldowns().iter().enumerate() {
                if let Some(skill) = self.students[i].lookup_skill(j) {
                    if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                        let caster = stat.uid();
                        let targets = self.resolve_targets(state, skill);

                        result.push(ActionContext::Use(Action {
                            caster,
                            targets,
                            skill,
                        }));
                    }
                }
            }
        }

        result
    }

    fn apply(&self, state: State, action: &core::actions::ActionContext) -> State {
        let action = match action {
            ActionContext::Wait => return state,
            ActionContext::Use(action) => action,
        };

        let mut state = state.clone();

        // 캐스터와 타깃을 각각 search_uid_mut으로 집으면 같은 State를 두 번 가변 대여하게 된다.
        // 한 번의 분할에서 갈라내고, 타깃은 거리 순서를 보존하도록 action.targets 순서로 담는다.
        let (boss, students) = state.split_mut();
        let mut caster = None;
        let mut slots: Vec<Option<&mut StateData>> =
            (0..action.targets.len()).map(|_| None).collect();

        for data in std::iter::once(boss).chain(students.iter_mut()) {
            let uid = data.uid();

            if uid == action.caster {
                caster = Some(data);
            } else if let Some(i) = action.targets.iter().position(|&target| target == uid) {
                slots[i] = Some(data);
            }
        }

        let mut targets: Vec<&mut StateData> = slots.into_iter().flatten().collect();
        action
            .skill
            .apply(caster.expect("unexpected uid"), &mut targets);

        state
    }

    fn advance(&self, state: &State, delta_ticks: u16) -> Result<State, error::Error> {
        let mut skill_mask = 0u64;

        for student in state.students() {
            skill_mask |= student.effects.data();
        }

        skill_mask |= state.boss().effects.data();

        let cost_per_second: u16 = self.cost_charge_time[&skill_mask.into()]; // TODO

        let boss_effects_len = state.boss().remained_effects().len();
        let boss_remain_effects_ref = &state.boss().remained_effects();
        let mut new_boss_remain_effects: BinaryHeap<Reverse<RemainedEffects>> =
            BinaryHeap::with_capacity(boss_effects_len);
        let mut boss_effects_mask = state.boss().effects();
        let mut boss_acc_damage = state.boss().accumulated_damage.clone();
        let damage = state.boss().damage_with_effects();
        for item in boss_remain_effects_ref {
            let bit = 1u64 << item.0.offset;

            if item.0.ticks <= delta_ticks {
                if damage.is_some() {
                    boss_acc_damage.push(AccumulatedDamage {
                        ticks: item.0.ticks,
                        damage: damage_map.get(boss_effects_mask),
                    });
                }
                boss_effects_mask &= !bit;
            } else {
                if damage.is_some() {
                    boss_acc_damage.push(AccumulatedDamage {
                        ticks: delta_ticks,
                        damage: damage_map.get(boss_effects_mask),
                    });
                }
                new_boss_remain_effects.push(Reverse(RemainedEffects {
                    ticks: item.0.ticks - delta_ticks,
                    offset: item.0.offset,
                }));
            }
        }

        let boss_effects = boss_effects_mask.into();

        let cooldowns_lambda = |t: &u16| t.saturating_sub(delta_ticks);

        let new_students: Vec<StateData> = state
            .students()
            .iter()
            .map(|student: &StateData| {
                let damage = student.damage_with_effects();
                let mut acc_damage = student.accumulated_damage.clone();

                let effects_len = student.remained_effects.len();
                let mut new_remain_effects = Vec::with_capacity(effects_len);
                let mut effects_mask = student.effects;
                for item in &student.remained_effects {
                    let bit = 1u64 << item.0.offset;

                    if item.0.ticks <= delta_ticks {
                        if damage.is_some() {
                            acc_damage.push(AccumulatedDamage {
                                ticks: item.0.ticks,
                                damage: damage_map.get(effects_mask),
                            });
                        }
                        effects_mask &= !bit;
                    } else {
                        if damage.is_some() {
                            acc_damage.push(AccumulatedDamage {
                                ticks: delta_ticks,
                                damage: damage_map.get(effects_mask),
                            });
                        }

                        let skill_type = self.lookup_skill(item.0.offset.into());
                        if let Ok(sk) = skill_type {
                            for skill_effect in sk.skill_effects() {
                                for target in skill_effect.targets {
                                    // 장판스킬일 경우 범위 안에 있는지 고려
                                    if let Land { kind, region } = target {
                                        if kind.is_other() {
                                            todo!()
                                        }
                                        let caster_state = state.state_data_by_id(sk.owner().id());
                                        if let Some(data) = caster_state
                                            && is_inside(
                                                student.coordinate,
                                                region,
                                                data.coordinate,
                                            )
                                        {
                                            new_remain_effects.push(RemainedEffects {
                                                ticks: item.0.ticks - delta_ticks,
                                                offset: item.0.offset,
                                            });
                                        }
                                    } else {
                                        new_remain_effects.push(RemainedEffects {
                                            ticks: item.0.ticks - delta_ticks,
                                            offset: item.0.offset,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                StateData::from_parts(
                    student.uid(),
                    student.coordinate,
                    student
                        .cooldowns
                        .iter()
                        .map(|i| i.saturating_sub(delta_ticks))
                        .collect(),
                    effects_mask.into(),
                    new_remain_effects,
                    acc_damage,
                    student.extra().as_ref().map(|s| s.clone_box()),
                )
            })
            .collect();

        let new_state = State::new(
            &new_students,
            state
                .boss()
                .clone_matching(cooldowns_lambda, boss_effects, new_boss_remain_effects),
            state.frames() + delta_ticks,
            (state.cost() + (delta_ticks * cost_per_second / TPS) as i8).min(10),
        );
        Ok(new_state)
    }

    fn next_event_frames(&self, state: &State) -> u16 {
        let mut result: u16 = u16::MAX;

        for student in state.students() {
            for (i, time) in student.cooldowns.iter().enumerate() {
                let cost = *time / self.cost_charge_time[&student.effects]; // TODO
                if student.character.skill_list()[i].cost() as u16 >= cost {
                    result = result.min(*time);
                }
            }
            let remain_effect = student.remained_effects.peek();
            if let Some(effect) = remain_effect {
                result = result.min(effect.0.ticks);
            }
        }

        for (i, time) in state.boss().cooldowns.iter().enumerate() {
            if state.boss().character.skill_list()[i].cost() as u16
                >= *time / self.cost_charge_time[&state.boss().effects]
            // TODO
            {
                result = result.min(*time);
            }
        }

        result
    }

    fn is_time_over(&self, ticks: u16) -> bool {
        self.limit_ticks <= ticks
    }

    fn lookup_skill(&self, index: usize) -> Result<&dyn Skill, error::Error> {
        let total_skill_count = 3 + self.students.len() * 3 + self.boss.skill_list().len();
        let student_skill_offset = 3;
        let boss_skill_offset = 3 + 3 * self.students.len();
        if index < 3 || index >= total_skill_count {
            return Err(error::Error::OutOfRange(format!(
                "{index} must be between 3 and {}",
                total_skill_count - 1
            )));
        }

        if index < boss_skill_offset {
            self.students
                .get((index - student_skill_offset) / 3)
                .ok_or(error::Error::Unknown(format!(
                    "index {} can't find skill",
                    index
                )))
                .unwrap()
                .skills
                .get((index - student_skill_offset) % 3)
                .ok_or(error::Error::Unknown(format!(
                    "index {} can't find skill",
                    index
                )))
                .map(|&x| &*x)
        } else {
            self.boss
                .skills
                .get(index - boss_skill_offset)
                .ok_or(error::Error::Unknown(format!(
                    "index {} can't find skill",
                    index
                )))
                .map(|&x| &*x)
        }
    }

    fn character_by_id(&self, id: u32) -> Option<Character> {
        if id == self.boss.uid() {
            Some(Character::Boss(&self.boss))
        } else {
            for student in &self.students {
                if id == student.uid() {
                    return Some(Character::Student(student));
                }
            }

            None
        }
    }
}
