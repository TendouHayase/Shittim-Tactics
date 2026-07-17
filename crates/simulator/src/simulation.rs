use core::{
    TPS,
    actions::ActionContext::{self},
    boss::{Boss, BossBehavior},
    damage::{
        Damage,
        key::{DamageKey, SkillsBitMask},
    },
    simulator::Simulator,
    skill::Skill,
    state::{AccumulatedDamage, RemainedEffects, State, StateData, Stateful},
    student::Student,
};
use std::{
    collections::{BinaryHeap, HashMap},
    rc::Rc,
    sync::Arc,
};

pub struct Simulation<'a, T: BossBehavior + Clone, const N: usize> {
    pub students: Vec<Student>,
    pub boss: Boss<T>,

    damage_list: HashMap<SkillsBitMask, Damage>,
    one_cost_charge_time_list: HashMap<SkillsBitMask, u16>,

    allocator: typed_arena::Arena<State<'a, N>>,
}

impl<T: BossBehavior + Clone, const N: usize> Simulator for Simulation<'_, T, N> {
    fn legal_actions<'a>(&self, state: &impl core::state::Stateful<'a>) -> Vec<Arc<dyn Skill>> {
        let cost = state.cost();
        let mut result = vec![];
        for (i, stat) in state.students().iter().enumerate() {
            for (j, cooltime) in stat.cooldowns.iter().enumerate() {
                let skill = self.students[i].skills[j].clone();
                if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                    result.push(skill);
                }
            }
        }

        result
    }

    fn apply<'a, 'b, 'c>(
        &self,
        state: &'b impl Stateful<'a>,
        action: &'b core::actions::ActionContext<dyn Skill + 'c>,
    ) -> Result<impl Stateful<'a>, error::Error> {
        let action = match action {
            ActionContext::Wait => return Ok((*state).clone()),
            ActionContext::Use(action) => action,
        };

        let caster_id = action.caster;

        let target_ids = &action.targets;

        let mut targets: Vec<&StateData> = Vec::with_capacity(target_ids.len());

        for id in target_ids {
            if *id == state.boss().character.id() {
                targets.push(state.boss());
            } else {
                for student in state.students() {
                    if *id == student.character.id() {
                        targets.push(student);
                    }
                }
            }
        }

        let mut new_state = state.clone();

        {
            {
                if caster_id == state.boss().character.id() {
                    let changed_target = action.skill.apply(new_state.boss(), &targets);
                    for target in changed_target {
                        if state.boss().character.id() == target.character.id() {
                            {
                                let boss_mut = new_state.boss_mut();
                                *boss_mut = target;
                            }
                        } else {
                            for student in new_state.students_mut() {
                                {
                                    if student.character.id() == target.character.id() {
                                        *student = target;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for student in state.students() {
                        if caster_id == student.character.id() {
                            for target in action.skill.apply(student, &targets) {
                                if state.boss().character.id() == target.character.id() {
                                    {
                                        let t = new_state.boss_mut();
                                        *t = target;
                                    }
                                } else {
                                    for student in new_state.students_mut() {
                                        if student.character.id() == target.character.id() {
                                            *student = target;
                                            break;

                                            *student = target;
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
            {}
        }

        Ok(new_state)
    }

    fn advance<'a, 'b>(
        &self,
        state: &'b impl Stateful<'a>,
        delta_ticks: u16,
    ) -> Result<impl Stateful<'a>, error::Error> {
        let mut skill_mask = 0u64;

        for student in state.students() {
            skill_mask |= student.effects.mask.data();
        }

        skill_mask |= state.boss().effects.mask.data();

        let cost_per_second: u16 = self.one_cost_charge_time_list[&skill_mask.into()].max(0) as u16;

        let boss_effects_len = state.boss().remained_effects.len();
        let boss_remain_effects_ref = &state.boss().remained_effects;
        let mut new_boss_remain_effects = BinaryHeap::with_capacity(boss_effects_len);
        let mut boss_effects_mask = state.boss().effects.clone().mask.data();
        let mut boss_acc_damage = state.boss().accumulated_damage.clone();
        let damage = state.boss().effects.damage();
        for item in boss_remain_effects_ref {
            let bit = 1u64 << item.bit;

            if item.ticks <= delta_ticks {
                if damage != None {
                    boss_acc_damage.push(AccumulatedDamage {
                        damage: DamageKey::from_mask(
                            boss_effects_mask.into(),
                            &state.boss().effects,
                        ),
                        ticks: item.ticks,
                    });
                }
                boss_effects_mask &= !bit;
            } else {
                if damage.is_some() {
                    boss_acc_damage.push(AccumulatedDamage {
                        damage: DamageKey::from_mask(
                            boss_effects_mask.into(),
                            &state.boss().effects,
                        ),
                        ticks: delta_ticks,
                    });
                }
                new_boss_remain_effects.push(RemainedEffects {
                    ticks: item.ticks - delta_ticks,
                    bit: item.bit,
                });
            }
        }

        let boss_effects = DamageKey::from_mask(boss_effects_mask.into(), &state.boss().effects);

        let cooldowns_lambda = |t: &u16| (t - delta_ticks).max(0);

        let mut student_effects_lambda = state.students().iter().map(|student: &StateData<'a>| {
            let damage = student.effects.damage();
            let mut acc_damage = student.accumulated_damage.clone();

            let effects_len = student.remained_effects.len();
            let mut new_remain_effects = BinaryHeap::with_capacity(effects_len);
            let mut effects_mask = student.effects.clone().mask.data();
            for item in &student.remained_effects {
                let bit = 1u64 << item.bit;

                if item.ticks <= delta_ticks {
                    if damage != None {
                        acc_damage.push(AccumulatedDamage {
                            damage: DamageKey::from_mask(effects_mask.into(), &student.effects),
                            ticks: item.ticks,
                        });
                    }
                    effects_mask &= !bit;
                } else {
                    if damage != None {
                        acc_damage.push(AccumulatedDamage {
                            damage: DamageKey::from_mask(effects_mask.into(), &student.effects),
                            ticks: delta_ticks,
                        });
                    }
                    new_remain_effects.push(RemainedEffects {
                        ticks: item.ticks - delta_ticks,
                        bit: item.bit,
                    });
                }
            }

            StateData {
                character: student.character,
                coordinate: student.coordinate,
                accumulated_damage_cache: student.accumulated_damage_cache.clone(),
                cooldowns: student
                    .cooldowns
                    .iter()
                    .map(|i| (i - delta_ticks).max(0))
                    .collect(),
                effects: DamageKey::from_mask(effects_mask.into(), &student.effects),
                remained_effects: new_remain_effects,
                accumulated_damage: acc_damage,
            }
        });

        let new_students: [StateData<'a>; N] =
            std::array::from_fn(|_| student_effects_lambda.next().unwrap());
        Ok(State {
            students: new_students,
            cost: (state.cost() + (delta_ticks * cost_per_second / TPS) as i8).min(10),
            boss: state.boss().clone_matching(
                cooldowns_lambda,
                boss_effects,
                new_boss_remain_effects,
            ),
            frames: state.frames() + delta_ticks,
        })
    }

    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16 {
        let mut result: u16 = u16::MAX;

        for student in state.students() {
            for i in &student.cooldowns {
                result = result.min(*i);
            }
        }

        for i in &state.boss().cooldowns {
            result = result.min(*i);
        }

        result
    }

    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage> {
        &self.damage_list
    }
}
