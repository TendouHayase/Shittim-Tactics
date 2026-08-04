use core::{
    actions::ActionContext::{self},
    boss::Boss,
    character::{Character, CharacterOps},
    damage::{Damage, key::SkillsBitMask},
    simulator::Simulator,
    skill::{EffectKind, Skill, SkillEffectTarget::Land, SkillOps},
    state::{AccumulatedDamage, RemainedEffects, StateData, Stateful},
    student::Student,
    utils::{TPS, is_inside},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fmt::Debug,
    marker::PhantomData,
    sync::Arc,
};

pub struct Simulation<'a, const N: usize, S: Stateful<'a>> {
    pub students: [Student; N],
    pub boss: Boss,

    limit_ticks: u16,

    damage_list: HashMap<SkillsBitMask, Damage>,
    cost_charge_time: HashMap<SkillsBitMask, u16>,

    _marker: PhantomData<&'a S>,
}

impl<const N: usize, S: for<'z> Stateful<'z>> Simulator for Simulation<'_, N, S> {
    type S<'a> = S;
    fn legal_actions<'a>(&self, state: &impl core::state::Stateful<'a>) -> Vec<&Skill> {
        let cost = state.cost();
        let mut result = vec![];
        for (i, stat) in state.students().iter().enumerate() {
            for (j, cooltime) in stat.cooldowns.iter().enumerate() {
                let skill = &self.students[i].skills[j];
                if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                    result.push(skill);
                }
            }
        }

        result
    }

    fn apply<'a>(
        &self,
        state: &Self::S<'a>,
        action: &core::actions::ActionContext,
    ) -> Self::S<'a> {
        let action = match action {
            ActionContext::Wait => return state.clone(),
            ActionContext::Use(action) => action,
        };

        let mut state = state.clone();

        let caster_id = action.caster;
        let target_ids = &action.targets;

        // 보스와 학생들을 한 번의 가변 대여로 동시에 분리한다.
        // 접근자를 따로 호출하면 같은 state를 두 번 가변 대여하게 되어 통과하지 못한다.
        let (boss, students) = state.split_mut();

        let mut targets: Vec<&mut StateData<'a>> = Vec::with_capacity(target_ids.len());

        // 각 가변 참조가 caster 또는 targets 중 정확히 한 곳으로만 이동하도록
        // 대상 목록을 단 한 번만 순회한다. id로 여러 번 조회하면 같은 대상을 두 번
        // 꺼낼 수 있다는 걸 컴파일러가 배제하지 못해 대여 검사를 통과할 수 없다.
        let caster = if caster_id == boss.character.id() {
            for student in students {
                if target_ids.contains(&student.character.id()) {
                    targets.push(student);
                }
            }

            boss
        } else {
            if target_ids.contains(&boss.character.id()) {
                targets.push(boss);
            }

            let mut caster = None;

            for student in students {
                let id = student.character.id();

                // 캐스터는 타깃 목록에 포함되지 않는다(`Simulator::apply` 문서 참고).
                if id == caster_id {
                    caster = Some(student);
                } else if target_ids.contains(&id) {
                    targets.push(student);
                }
            }

            caster.expect("caster id does not match any character in the state")
        };

        action.skill.apply(caster, &mut targets);

        state
    }

    fn advance<'a>(
        &self,
        state: &Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error> {
        let mut skill_mask = 0u64;

        let damage_map = state.boss().damage_map;

        for student in state.students() {
            skill_mask |= student.effects.data();
        }

        skill_mask |= state.boss().effects.data();

        let cost_per_second: u16 = self.cost_charge_time[&skill_mask.into()];

        let boss_effects_len = state.boss().remained_effects.len();
        let boss_remain_effects_ref = &state.boss().remained_effects;
        let mut new_boss_remain_effects: BinaryHeap<Reverse<RemainedEffects>> =
            BinaryHeap::with_capacity(boss_effects_len);
        let mut boss_effects_mask = state.boss().effects.clone().data();
        let mut boss_acc_damage = state.boss().accumulated_damage.clone();
        let damage = state.boss().damage_with_effects();
        for item in boss_remain_effects_ref {
            let bit = 1u64 << item.0.offset;

            if item.0.ticks <= delta_ticks {
                if damage.is_some() {
                    boss_acc_damage.push(AccumulatedDamage {
                        ticks: item.0.ticks,
                        damage: damage_map.get(&boss_effects_mask.into()).copied(),
                    });
                }
                boss_effects_mask &= !bit;
            } else {
                if damage.is_some() {
                    boss_acc_damage.push(AccumulatedDamage {
                        ticks: delta_ticks,
                        damage: damage_map.get(&boss_effects_mask.into()).copied(),
                    });
                }
                new_boss_remain_effects.push(Reverse(RemainedEffects {
                    ticks: item.0.ticks - delta_ticks,
                    offset: item.0.offset,
                }));
            }
        }

        let boss_effects = boss_effects_mask.into();

        let cooldowns_lambda = |t: &u16| t - delta_ticks;

        let mut student_effects_lambda = state.students().iter().map(|student: &StateData<'a>| {
            let damage = student.damage_with_effects();
            let mut acc_damage = student.accumulated_damage.clone();

            let effects_len = student.remained_effects.len();
            let mut new_remain_effects: BinaryHeap<Reverse<RemainedEffects>> =
                BinaryHeap::with_capacity(effects_len);
            let mut effects_mask = student.effects.clone().data();
            for item in &student.remained_effects {
                let bit = 1u64 << item.0.offset;

                if item.0.ticks <= delta_ticks {
                    if damage.is_some() {
                        acc_damage.push(AccumulatedDamage {
                            ticks: item.0.ticks,
                            damage: damage_map.get(&effects_mask.into()).copied(),
                        });
                    }
                    effects_mask &= !bit;
                } else {
                    if damage.is_some() {
                        acc_damage.push(AccumulatedDamage {
                            ticks: delta_ticks,
                            damage: damage_map.get(&effects_mask.into()).copied(),
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
                                        && is_inside(student.coordinate, region, data.coordinate)
                                    {
                                        new_remain_effects.push(Reverse(RemainedEffects {
                                            ticks: item.0.ticks - delta_ticks,
                                            offset: item.0.offset,
                                        }));
                                    }
                                } else {
                                    new_remain_effects.push(Reverse(RemainedEffects {
                                        ticks: item.0.ticks - delta_ticks,
                                        offset: item.0.offset,
                                    }));
                                }
                            }
                        }
                    }
                }
            }

            StateData {
                character: student.character,
                coordinate: student.coordinate,
                accumulated_damage_cache: student.accumulated_damage_cache.clone(),
                cooldowns: student.cooldowns.iter().map(|i| i - delta_ticks).collect(),
                effects: effects_mask.into(),
                remained_effects: new_remain_effects,
                accumulated_damage: acc_damage,
                damage_map,
                extra: student.extra,
            }
        });

        let new_students: [StateData<'a>; N] =
            std::array::from_fn(|_| student_effects_lambda.next().unwrap());
        let new_state = Self::S::new(
            &new_students,
            state
                .boss()
                .clone_matching(cooldowns_lambda, boss_effects, new_boss_remain_effects),
            state.frames() + delta_ticks,
            (state.cost() + (delta_ticks * cost_per_second / TPS) as i8).min(10),
        );
        Ok(new_state)
    }

    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16 {
        let mut result: u16 = u16::MAX;

        for student in state.students() {
            for (i, time) in student.cooldowns.iter().enumerate() {
                let cost = *time / self.cost_charge_time[&student.effects];
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

    fn lookup_skill(&self, index: usize) -> Result<&Skill, error::Error> {
        let total_skill_count = 3 + N * 3 + self.boss.skill_list().len();
        let student_skill_offset = 3;
        let boss_skill_offset = 3 + 3 * N;
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
        } else {
            self.boss
                .skills
                .get(index - boss_skill_offset)
                .ok_or(error::Error::Unknown(format!(
                    "index {} can't find skill",
                    index
                )))
        }
    }

    fn character_by_id(&self, id: u32) -> Option<Character> {
        if id == self.boss.id() {
            Some(Character::Boss(&self.boss))
        } else {
            for student in &self.students {
                if id == student.id() {
                    return Some(Character::Student(student));
                }
            }

            None
        }
    }
}
