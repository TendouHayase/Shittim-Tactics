use std::sync::Arc;

use error::Error;

use crate::{
    actions::ActionContext,
    boss::Boss,
    character::Character,
    constants::TPS,
    skill::{Skill, SkillEffectTarget},
    state::{State, StateData, StudentState},
    student::Student,
    uid::{SkillUid, Uid},
    utils::{Position, euclidean_distance, is_inside},
};

pub struct Simulator {
    pub students: Vec<Student>,
    pub boss: Boss,
    pub skills: Vec<Arc<dyn Skill>>,

    limit_ticks: u16,

    cost_per_second: u16,
}

impl Simulator {
    fn initial_state(&self) -> State {
        // 학생 스킬수는 항상 3
        if self.students.len() == 6 {
            State {
                students: StudentState::TotalAssault(std::array::from_fn(|i| {
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
                students: StudentState::FinalRestrictionRelease(std::array::from_fn(|i| {
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

    fn legal_actions(&self, state: &State) -> Vec<ActionContext> {
        let cost = state.cost();
        let mut result = vec![];
        for (i, stat) in state.students().iter().enumerate() {
            for (j, cooltime) in stat.cooldowns().iter().enumerate() {
                if let Some(skill) = self.students[i].lookup_skill(j)
                    && *cooltime == 0
                    && cost >= skill.cost().try_into().unwrap()
                {
                    let caster = stat.uid();
                    let targets = self.resolve_targets(state, skill);

                    result.push(ActionContext {
                        caster,
                        targets,
                        skill: skill.uid(),
                    });
                }
            }
        }

        result
    }

    fn resolve_targets(&self, state: &State, skill: &dyn Skill) -> Vec<Uid> {
        let caster_id = skill.owner();
        let caster_coord = state
            .search_uid(caster_id)
            .map(|data| data.coordinate())
            .unwrap_or_default();

        let mut targets = Vec::new();

        for skill_effect in skill.skill_effects() {
            let target = skill_effect.targets;
            match target {
                // 캐스터 자신에 대한 효과는 `Skill::apply`의 caster 인자로 처리
                SkillEffectTarget::Oneself { .. } => {}

                SkillEffectTarget::Student { count, .. } => {
                    let mut students: Vec<(Position, Uid)> = state
                        .students()
                        .iter()
                        .map(|student| (student.coordinate(), student.uid()))
                        .filter(|student| student.1 != caster_id)
                        .collect();

                    // 유클리드 거리로 정렬
                    students.sort_by(|lhs, rhs| {
                        euclidean_distance(caster_coord, lhs.0)
                            .total_cmp(&euclidean_distance(caster_coord, rhs.0))
                    });

                    // 캐스터를 뺀 인원이 count보다 적을 수 있으므로 인덱싱 대신 take.
                    targets.extend(students.iter().take(count.into()).map(|s| s.1));
                }

                SkillEffectTarget::Boss { .. } => targets.push(state.boss().uid()),

                SkillEffectTarget::Land { region, .. } => {
                    if is_inside(state.boss().coordinate(), region, caster_coord) {
                        targets.push(state.boss().uid());
                    }

                    for student in state.students() {
                        if is_inside(student.coordinate(), region, caster_coord) {
                            targets.push(student.uid());
                        }
                    }
                }
            }
        }

        targets
    }

    fn apply(&self, mut state: State, action: &ActionContext) -> Result<State, Error> {
        // 타깃은 거리 순서를 보존하도록 action.targets 순서로 담음.

        let (boss, students) = state.split_mut();
        let caster;
        let mut targets: Vec<&mut StateData>;

        let caster_uid = action.caster;

        if boss.uid() == caster_uid {
            caster = boss;

            targets = students
                .iter_mut()
                .filter(|s| action.targets.contains(&s.uid()))
                .collect();
        } else {
            let caster_idx = students
                .iter()
                .position(|s| s.uid() == caster_uid)
                .ok_or(Error::NotFound)?;

            let (left, right) = students.split_at_mut(caster_idx);
            let other;
            (caster, other) = right.split_first_mut().unwrap(); // 위에서 캐스터 존재여부 검사함

            targets = left
                .iter_mut()
                .chain(other.iter_mut())
                .filter(|s| action.targets.contains(&s.uid()))
                .collect();
        }
        let skill = self.lookup_skill(action.skill).ok_or(Error::NotFound)?;
        skill.apply(caster, &mut targets);

        Ok(state)
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
            if self.cost_per_second != 0 {
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
            if self.cost_per_second != 0
                && boss.skills()[i].cost() as u16 >= *time / self.cost_per_second
            {
                result = result.min(*time);
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

    fn lookup_skill(&self, uid: SkillUid) -> Option<&dyn Skill> {
        self.skills.get(uid.skill_index()).map(|skill| &**skill)
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
