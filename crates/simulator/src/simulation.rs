use core::{
    actions::{
        Action,
        ActionContext::{self},
    },
    boss::Boss,
    character::Character,
    constants::TPS,
    simulator::Simulator,
    skill::Skill,
    state::{State, StateData},
    student::Student,
    uid::Uid,
};
use std::sync::Arc;

use error::Error;

pub struct Simulation {
    pub students: Vec<Student>,
    pub boss: Boss,
    pub skills: Vec<Arc<dyn Skill>>,

    limit_ticks: u16,

    cost_per_second: u16,
}

impl Simulator for Simulation {
    fn initial_state(&self) -> State {
        if self.students.len() == 6 {
            State {
                students: core::state::StudentState::TotalAssault(std::array::from_fn(|i| {
                    StateData::new(
                        self.students[i].uid(),
                        3,
                        self.students[i].extra.map(|init| init()),
                    )
                })),
                boss: StateData::new(
                    self.boss.uid(),
                    self.boss.skills.len(),
                    self.boss.extra.map(|init| init()),
                ),
                frames: 0,
                cost: 0,
            }
        } else if self.students.len() == 10 {
            State {
                students: core::state::StudentState::TotalAssault(std::array::from_fn(|i| {
                    StateData::new(
                        self.students[i].uid(),
                        3,
                        self.students[i].extra.map(|init| init()),
                    )
                })),
                boss: StateData::new(
                    self.boss.uid(),
                    self.boss.skills.len(),
                    self.boss.extra.map(|init| init()),
                ),
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

        // 타깃은 거리 순서를 보존하도록 action.targets 순서로 담음.
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

    fn advance(&self, state: &State, delta_ticks: u16) -> Result<State, Error> {
        let mut next = state.clone();
        let (boss, students) = next.split_mut();

        for data in std::iter::once(boss).chain(students.iter_mut()) {
            for cooldown in data.cooldowns_mut() {
                *cooldown = cooldown.saturating_sub(delta_ticks);
            }

            // 효과 구간이 [적용, 적용 + 지속)이라 남은 틱이 delta와 같으면 이번에 끝난다.
            let effects = data.remained_effects_mut();
            effects.retain(|effect| effect.ticks > delta_ticks);
            for effect in effects.iter_mut() {
                effect.ticks -= delta_ticks;
            }
        }

        // 데미지 누적은 효과별 틱 계산이 들어올 때까지 빔.

        // next_event_frames가 u16::MAX를 돌려줄 수 있어 u16으로 곱하거나 더하면 넘침.
        let gained = (delta_ticks as u32 * self.cost_per_second as u32 / TPS as u32).min(10) as i8;
        next.frames = next.frames.saturating_add(delta_ticks);
        next.cost = (next.cost + gained).min(10);

        Ok(next)
    }

    fn next_event_frames(&self, state: &State) -> u16 {
        let mut result: u16 = u16::MAX;

        for student in state.students() {
            // 논리적으로 uid 항상 존재
            let character = self.character_by_uid(student.uid()).unwrap();
            if (self.cost_per_second != 0) {
                for (i, time) in student.cooldowns().iter().enumerate() {
                    let cost = *time / self.cost_per_second;
                    if character.skills()[i].cost() as u16 >= cost {
                        result = result.min(*time);
                    }
                }
            }

            for effect in student.remained_effects() {
                result = result.min(effect.ticks);
            }
        }

        // 논리적으로 uid 항상 존재
        let boss = self.character_by_uid(state.boss().uid()).unwrap();
        for (i, time) in state.boss().cooldowns().iter().enumerate() {
            if (self.cost_per_second != 0) {
                if boss.skills()[i].cost() as u16 >= *time / self.cost_per_second {
                    result = result.min(*time);
                }
            }

            for effect in state.boss().remained_effects() {
                result = result.min(effect.ticks);
            }
        }
        result
    }

    fn is_time_over(&self, ticks: u16) -> bool {
        self.limit_ticks <= ticks
    }

    fn lookup_skill(&self, index: usize) -> Option<&dyn Skill> {
        self.skills.get(index).map(|skill| &**skill)
    }

    fn character_by_uid(&self, uid: Uid) -> Option<&dyn Character> {
        if uid == self.boss.uid() {
            Some(&self.boss)
        } else {
            for student in &self.students {
                if uid == student.uid() {
                    return Some(student);
                }
            }
            None
        }
    }
}
