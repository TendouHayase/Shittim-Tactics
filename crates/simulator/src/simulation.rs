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
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    rc::Rc,
    sync::Arc,
};

pub struct Simulation<'a, T: BossBehavior + Clone, const N: usize> {
    pub students: Vec<Student>,
    pub boss: Boss<T>,

    limit_ticks: u16,

    damage_list: HashMap<SkillsBitMask, Damage>,
    cost_charge_time: HashMap<SkillsBitMask, u16>,

    allocator: typed_arena::Arena<State<'a, N>>,
}

impl<T: BossBehavior + Clone, const N: usize> Simulator for Simulation<'_, T, N> {
    type S<'a> = State<'a, N>;
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

    fn apply<'a: 'b, 'b, 'c>(
        &self,
        state: &'b Self::S<'a>,
        action: &'b core::actions::ActionContext<dyn Skill + 'c>,
    ) -> Self::S<'a> {
        let action = match action {
            ActionContext::Wait => return (*state).clone(),
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
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        new_state
    }

    fn advance<'a: 'b, 'b>(
        &self,
        state: &'b Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error> {
        let mut skill_mask = 0u64;

        for student in state.students() {
            skill_mask |= student.effects.mask.data();
        }

        skill_mask |= state.boss().effects.mask.data();

        let cost_per_second: u16 = self.cost_charge_time[&skill_mask.into()].max(0) as u16;

        let boss_effects_len = state.boss().remained_effects.len();
        let boss_remain_effects_ref = &state.boss().remained_effects;
        let mut new_boss_remain_effects: BinaryHeap<Reverse<RemainedEffects>> =
            BinaryHeap::with_capacity(boss_effects_len);
        let mut boss_effects_mask = state.boss().effects.clone().mask.data();
        let mut boss_acc_damage = state.boss().accumulated_damage.clone();
        let damage = state.boss().effects.damage();
        for item in boss_remain_effects_ref {
            let bit = 1u64 << item.0.bit;

            if item.0.ticks <= delta_ticks {
                if damage != None {
                    boss_acc_damage.push(AccumulatedDamage {
                        damage: DamageKey::from_mask(
                            boss_effects_mask.into(),
                            &state.boss().effects,
                        ),
                        ticks: item.0.ticks,
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
                new_boss_remain_effects.push(Reverse(RemainedEffects {
                    ticks: item.0.ticks - delta_ticks,
                    bit: item.0.bit,
                }));
            }
        }

        let boss_effects = DamageKey::from_mask(boss_effects_mask.into(), &state.boss().effects);

        let cooldowns_lambda = |t: &u16| (t - delta_ticks).max(0);

        let mut student_effects_lambda = state.students().iter().map(|student: &StateData<'a>| {
            let damage = student.effects.damage();
            let mut acc_damage = student.accumulated_damage.clone();

            let effects_len = student.remained_effects.len();
            let mut new_remain_effects: BinaryHeap<Reverse<RemainedEffects>> =
                BinaryHeap::with_capacity(effects_len);
            let mut effects_mask = student.effects.clone().mask.data();
            for item in &student.remained_effects {
                let bit = 1u64 << item.0.bit;

                if item.0.ticks <= delta_ticks {
                    if damage != None {
                        acc_damage.push(AccumulatedDamage {
                            damage: DamageKey::from_mask(effects_mask.into(), &student.effects),
                            ticks: item.0.ticks,
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
                    new_remain_effects.push(Reverse(RemainedEffects {
                        ticks: item.0.ticks - delta_ticks,
                        bit: item.0.bit,
                    }));
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
            for (i, time) in student.cooldowns.iter().enumerate() {
                let cost = *time / self.cost_charge_time[&student.effects.mask];
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
                >= *time / self.cost_charge_time[&state.boss().effects.mask]
            {
                result = result.min(*time);
            }
        }

        result
    }

    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage> {
        &self.damage_list
    }

    fn is_time_over(&self, ticks: u16) -> bool {
        self.limit_ticks <= ticks
    }
}
