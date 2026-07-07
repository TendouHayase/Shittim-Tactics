use core::{
    TPS,
    actions::{
        Action,
        ActionContext::{self},
    },
    boss::{Boss, BossState, BossTrait},
    simulator::Simulator,
    skill::{
        Buff, CasterContext, Debuff, Effect, EffectKind, EffectTiming, Skill, SkillContext,
        TargetContext,
    },
    state::{
        State::{self, Assault, RestrictionRelease},
        StateData,
    },
    student::{Student, StudentState},
};
use std::{collections::LinkedList, rc::Rc};

use allocator::pool::PoolAllocator;

pub struct Simulation<'a: 'b, 'b, T: BossTrait> {
    pub students: Vec<Student>,
    pub boss: Boss<T>,

    pub g_cost: f64,
    pub score: f64,

    pub allocator: PoolAllocator<'a, State<'b, T>>,
}

impl<'b, 'c, T: BossTrait + Clone> Simulator<T> for Simulation<'_, 'b, T> {
    fn legal_actions(&self, state: &core::state::State<T>) -> Vec<Rc<dyn Skill>> {
        match state {
            Assault(content) => {
                let cost = content.cost;
                let mut result = vec![];
                for (i, stat) in content.students.iter().enumerate() {
                    for (j, cooltime) in stat.cooldowns.iter().enumerate() {
                        let skill = self.students[i].skills[j].clone();
                        if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                            result.push(skill);
                        }
                    }
                }

                result
            }
            RestrictionRelease(content) => {
                let cost = content.cost;
                let mut result = vec![];
                for (i, stat) in content.students.iter().enumerate() {
                    for (j, cooltime) in stat.cooldowns.iter().enumerate() {
                        let skill = self.students[i].skills[j].clone();
                        if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                            result.push(skill);
                        }
                    }
                }

                result
            }
        }
    }

    fn apply<'a>(
        &self,
        state: &'a State<T>,
        action: &core::actions::ActionContext<dyn Skill>,
    ) -> core::state::State<'a, T> {
        let action = match action {
            ActionContext::Wait => return state.clone(),
            ActionContext::Use(action) => action,
        };

        match state {
            Assault(data) => {
                let mut new_data = data.clone();
                Simulation::apply_action_to_data(&mut new_data, action);
                State::Assault(data.clone())
            }
            RestrictionRelease(data) => {
                let mut new_data = data.clone();
                Simulation::apply_action_to_data(&mut new_data, action);
                State::RestrictionRelease(data.clone())
            }
        }
    }

    fn advance<'a>(&'a self, state: &'a State<T>, delta_ticks: u32) -> State<'a, T> {
        let mut boss_state = state.boss();
        let students_state = state.students();

        let cost_buff_amount: i32 = students_state
            .iter()
            .map(|student| {
                let result: i32 = student
                    .effects
                    .iter()
                    .map(|effect| match &effect.kind {
                        EffectKind::Buff {
                            ty,
                            duration: _,
                            scale: _,
                            amount,
                        } => {
                            if *ty == Buff::CostRecovery {
                                *amount as i32
                            } else {
                                0
                            }
                        }
                        EffectKind::Debuff {
                            ty,
                            duration: _,
                            scale: _,
                            amount,
                        } => {
                            if *ty == Debuff::CostRecovery {
                                -1 * *amount as i32
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    })
                    .sum();
                result
            })
            .sum();
        let cost_buff_scale: i32 = students_state
            .iter()
            .map(|student| {
                let result: i32 = student
                    .effects
                    .iter()
                    .map(|effect| match &effect.kind {
                        EffectKind::Buff {
                            ty,
                            duration: _,
                            scale,
                            amount: _,
                        } => {
                            if *ty == Buff::CostRecovery {
                                *scale as i32
                            } else {
                                0
                            }
                        }
                        EffectKind::Debuff {
                            ty,
                            duration: _,
                            scale,
                            amount: _,
                        } => {
                            if *ty == Debuff::CostRecovery {
                                -1 * *scale as i32
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    })
                    .sum();
                result
            })
            .sum();

        let cost_per_second: u32 = ((students_state
            .iter()
            .map(|student| student.student.stats.base_stats.cost_recovery as i32)
            .sum::<i32>()
            + cost_buff_amount)
            * cost_buff_scale
            / 10000)
            .max(0) as u32;

        boss_state.effects = boss_state
            .effects
            .extract_if(|effect| {
                effect.timing == EffectTiming::Instant
                    || match &effect.kind {
                        EffectKind::Buff {
                            ty: _,
                            duration,
                            scale: _,
                            amount: _,
                        } => *duration <= delta_ticks,
                        EffectKind::Debuff {
                            ty: _,
                            duration,
                            scale: _,
                            amount: _,
                        } => *duration <= delta_ticks,
                        _ => false,
                    }
            })
            .collect();

        match state {
            State::Assault(data) => {
                let mut new_student_state = data.students.clone();
                let _ = new_student_state.iter_mut().map(|student| {
                    student.effects.extract_if(|effect| {
                        effect.timing == EffectTiming::Instant
                            || match &effect.kind {
                                EffectKind::Buff {
                                    ty: _,
                                    duration,
                                    scale: _,
                                    amount: _,
                                } => *duration <= delta_ticks,
                                EffectKind::Debuff {
                                    ty: _,
                                    duration,
                                    scale: _,
                                    amount: _,
                                } => *duration <= delta_ticks,
                                _ => false,
                            }
                    })
                });

                State::Assault(StateData {
                    students: new_student_state,
                    cost: data.cost + (delta_ticks * cost_per_second / TPS).min(10) as i8,
                    boss: boss_state,
                })
            }
            State::RestrictionRelease(data) => {
                let mut new_student_state = data.students.clone();
                for new_student in new_student_state.iter_mut() {
                    new_student.effects = new_student
                        .effects
                        .extract_if(|effect| {
                            effect.timing == EffectTiming::Instant
                                || match &effect.kind {
                                    EffectKind::Buff {
                                        ty: _,
                                        duration,
                                        scale: _,
                                        amount: _,
                                    } => *duration <= delta_ticks,
                                    EffectKind::Debuff {
                                        ty: _,
                                        duration,
                                        scale: _,
                                        amount: _,
                                    } => *duration <= delta_ticks,
                                    _ => false,
                                }
                        })
                        .collect::<LinkedList<Effect>>();
                }

                State::RestrictionRelease(StateData {
                    students: new_student_state,
                    cost: data.cost + (delta_ticks * cost_per_second / TPS).min(10) as i8,
                    boss: boss_state,
                })
            }
        }
    }

    fn next_event_frames(&self, state: &State<T>) -> u32 {
        let mut result: u32 = u32::MAX;

        for student in state.students() {
            result = *student
                .cooldowns
                .iter()
                .min()
                .unwrap_or(&u32::MAX)
                .min(&result);
        }

        result = *state
            .boss()
            .cooldowns
            .iter()
            .min()
            .unwrap_or(&u32::MAX)
            .min(&result);

        result
    }
}

impl<T: BossTrait> Simulation<'_, '_, T> {
    pub fn build_skill_context<'a, const N: usize>(
        data: &'a mut StateData<'a, N, T>,
        action: &'a Action<'a, dyn Skill>,
    ) -> SkillContext<'a> {
        let caster_id = action.caster.id();
        let caster_idx = Self::find_index::<N>(&data.students, caster_id);

        let target_ids: Vec<u32> = action.targets.iter().map(|t| t.id()).collect();

        let mut seen = std::collections::HashSet::new();
        seen.insert(caster_idx);
        for &id in &target_ids {
            if id != data.boss.boss.stats.id {
                let idx = Self::find_index(&data.students, id);
                assert!(
                    seen.insert(idx),
                    "duplicate or self-targeting index detected; handle separately"
                );
            }
        }

        let ptr = data.students.as_mut_ptr();
        let caster_student = unsafe { &mut *ptr.add(caster_idx) };
        let caster_ctx = CasterContext::from_student(
            caster_student.student,
            caster_student,
            action.skill.skill_type(),
        );

        let mut target_contexts = Vec::with_capacity(target_ids.len());
        for &id in &target_ids {
            if id == data.boss.boss.stats.id {
                let boss_ref: &'a mut BossState<'a, T> =
                    unsafe { &mut *(&mut data.boss as *mut BossState<'a, T>) };
                target_contexts.push(TargetContext::from_boss(boss_ref.boss, boss_ref));
            } else {
                let idx = Self::find_index(&data.students, id);
                let student_ref: &'a mut StudentState = unsafe { &mut *ptr.add(idx) };
                target_contexts.push(TargetContext::from_student(
                    student_ref.student,
                    student_ref,
                ));
            }
        }

        SkillContext {
            name: action.skill.name(),
            caster: caster_ctx,
            targets: target_contexts,
        }
    }

    fn apply_action_to_data<'a, const N: usize>(
        data: &'a mut StateData<'a, N, T>,
        action: &'a Action<dyn Skill>,
    ) {
        let mut skill_context = Self::build_skill_context(data, action);

        action
            .skill
            .apply(&mut skill_context.caster, &mut skill_context.targets);
    }

    fn find_index<const N: usize>(students: &[StudentState; N], id: u32) -> usize {
        students
            .iter()
            .position(|s| s.student.stats.student_stats.id == id)
            .expect("caster or target id not found in current state")
    }
}
