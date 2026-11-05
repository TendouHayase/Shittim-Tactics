use std::{collections::VecDeque, sync::Arc};

use error::Error;

use crate::{
    actions::ActionContext,
    boss::Boss,
    character::Character,
    constants::{COST_UNIT, MAX_COST},
    damage::{Damage, DamageMode},
    effect::{Effect, EffectTiming},
    skill::{Region, Skill, SkillEffect, SkillEffectTarget, SkillType},
    state::{PendingCast, PendingHit, State, StateData},
    student::Student,
    uid::{SkillUid, Uid},
    utils::{Position, euclidean_distance, is_inside},
};

pub struct Simulator {
    pub students: Vec<Student>,
    pub boss: Boss,
    pub skills: Vec<Arc<dyn Skill>>,

    limit_ticks: u16,

    // 10000이 초당 1코스트.
    cost_per_second: u32,

    damage_mode: DamageMode,

    initial_skill_queue: VecDeque<SkillUid>,

    min_input_interval: u16,
}

impl Simulator {
    /// Skills hold a `Weak<Simulator>`, so build through [`Arc::new_cyclic`] and hand the weak
    /// reference to the skills from inside its closure.
    ///
    /// `initial_skill_queue` is the player's chosen Ex skill order, front of the queue first.
    /// `min_input_interval` is the fewest ticks a player needs between two Ex skills; 0 disables
    /// the limit and yields frame-perfect timelines.
    pub fn new(
        students: Vec<Student>,
        boss: Boss,
        skills: Vec<Arc<dyn Skill>>,
        limit_ticks: u16,
        damage_mode: DamageMode,
        initial_skill_queue: Vec<SkillUid>,
        min_input_interval: u16,
    ) -> Self {
        let cost_per_second = students
            .iter()
            .map(|student| student.stats().cost_recovery as u32)
            .sum();

        Self {
            students,
            boss,
            skills,
            limit_ticks,
            cost_per_second,
            damage_mode,
            initial_skill_queue: initial_skill_queue.into(),
            min_input_interval,
        }
    }

    pub fn initial_state(&self) -> State {
        let students: Vec<StateData> = self
            .students
            .iter()
            .map(|student| {
                // 학생 스킬수는 항상 3
                StateData::new(
                    student.uid(),
                    student.stats().hp,
                    3,
                    student.extra.map(|init| init()),
                )
            })
            .collect();
        let boss = StateData::new(
            self.boss.uid(),
            self.boss.stats().hp,
            self.boss.skills.len(),
            self.boss.extra.map(|init| init()),
        );

        State::new(&students, boss, 0, 0, self.initial_skill_queue.clone())
    }

    pub fn legal_actions(&self, state: &State) -> Vec<ActionContext> {
        self.hand(state)
            .filter(|skill| self.ticks_until_ready(state, *skill) == Some(0))
            .map(|skill| ActionContext {
                caster: skill.owner_uid(),
                targets: resolve_targets(state, skill),
                skill: skill.uid(),
            })
            .collect()
    }

    pub fn apply(&self, mut state: State, action: &ActionContext) -> Result<State, Error> {
        let skill = self.lookup_skill(action.skill).ok_or(Error::NotFound)?;
        let caster = self
            .character_by_uid(action.caster)
            .ok_or(Error::NotFound)?;

        if skill.skill_type() == SkillType::Ex && caster.uid() != self.boss.uid() {
            play_ex_skill(&mut state, skill, caster, self.min_input_interval)?;
        }

        // 효과와 데미지는 시전 시간이 지나야 들어간다.
        state.schedule_cast(PendingCast {
            frame: state.frames().saturating_add(skill.frames()),
            action: action.clone(),
        });

        // 시전 시간이 0이면 다음 advance를 기다리지 않는다.
        let now = state.frames();
        state = self.land_casts(state, now)?;
        state.settle_hits();

        Ok(state)
    }

    fn land_casts(&self, mut state: State, until: u16) -> Result<State, Error> {
        let now = state.frames();

        // 시전 완료가 새 시전을 예약할 수 있어 하나씩 꺼낸다.
        while let Some(cast) = state.pop_cast_due(until) {
            // 효과의 시각이 시전 완료 기준으로 잡히고, 그 전 타격을 볼 수 있도록 시각을 맞춘다.
            state.frames = cast.frame;
            state.settle_hits();
            state = self.land_cast(state, &cast.action)?;
        }

        state.frames = now;
        Ok(state)
    }

    fn land_cast(&self, state: State, action: &ActionContext) -> Result<State, Error> {
        let skill = self.lookup_skill(action.skill).ok_or(Error::NotFound)?;
        let caster = self
            .character_by_uid(action.caster)
            .ok_or(Error::NotFound)?;

        // 타깃은 거리 순서를 보존하도록 action.targets 순서로 담음.
        let targets: Vec<&dyn Character> = action
            .targets
            .iter()
            .map(|&tgt| self.character_by_uid(tgt).ok_or(Error::NotFound))
            .collect::<Result<Vec<&dyn Character>, Error>>()?;

        let mut state = skill.apply(state, caster, &targets);
        self.schedule_boss_hits(&mut state, skill, caster);

        Ok(state)
    }

    pub fn advance(&self, state: &State, delta_ticks: u16) -> Result<State, Error> {
        // next_event_frames가 u16::MAX를 돌려줄 수 있어 그냥 더하면 넘침.
        let end_tick = state.frames.saturating_add(delta_ticks);

        // 구간 안에 시작하고 끝나는 효과도 만료되도록 tick_down보다 먼저 적용한다.
        let mut next = self.land_casts(state.clone(), end_tick)?;

        let (boss, students) = next.split_mut();
        for data in std::iter::once(boss).chain(students.iter_mut()) {
            tick_down(data, delta_ticks, end_tick);
        }

        next.frames = end_tick;
        next.cost = regenerated_cost(next.cost, delta_ticks, self.cost_per_second);
        next.settle_hits();

        Ok(next)
    }

    /// Ticks until the next skill in the hand becomes usable, or `u16::MAX` if none ever will.
    pub fn next_event_frames(&self, state: &State) -> u16 {
        self.hand(state)
            .filter_map(|skill| self.ticks_until_ready(state, skill))
            // 이미 사용 가능한 스킬까지 세면 대기 행동의 시간이 흐르지 않는다.
            .filter(|&ticks| ticks > 0)
            .min()
            .unwrap_or(u16::MAX)
    }

    pub fn is_time_over(&self, ticks: u16) -> bool {
        self.limit_ticks <= ticks
    }

    fn hand<'a>(&'a self, state: &'a State) -> impl Iterator<Item = &'a dyn Skill> + 'a {
        state.hand().filter_map(|uid| self.lookup_skill(uid))
    }

    fn ticks_until_ready(&self, state: &State, skill: &dyn Skill) -> Option<u16> {
        let owner = self.character_by_uid(skill.owner_uid())?;
        let data = state.search_uid(skill.owner_uid())?;
        let cooldown = *data.cooldowns().get(cooldown_slot(owner, skill)?)?;
        let input_wait = state.next_input_frame.saturating_sub(state.frames());

        ticks_until_ready(cooldown, skill.cost(), state.cost(), self.cost_per_second)
            .map(|ticks| ticks.max(input_wait))
    }

    // 스킬의 데미지는 여기서 일괄 예약하므로 Skill::apply에서 다시 넣으면 이중 계산된다.
    fn schedule_boss_hits(&self, state: &mut State, skill: &dyn Skill, caster: &dyn Character) {
        // 보스 분포만 추적한다. 학생 체력은 회복 상한과 사망 때문에 단순 합성곱이 아니다.
        if caster.uid() == self.boss.uid() {
            return;
        }

        let caster_coord = coordinate_of(state, caster.uid());
        let boss_coord = state.boss().coordinate();
        let first_frame = state.frames();

        for skill_effect in skill.skill_effects() {
            let Some((coef_num, coef_den)) = boss_hit_coef(skill_effect, boss_coord, caster_coord)
            else {
                continue;
            };

            let damage =
                Damage::attack_damage(caster, &self.boss, coef_num as u64, coef_den as u64);
            let damage = self.damage_mode.resolve(damage);

            for frame in hit_frames(skill_effect.timing, first_frame) {
                state.schedule_hit(PendingHit {
                    frame,
                    damage,
                    source: skill.uid(),
                });
            }
        }
    }

    pub fn lookup_skill(&self, uid: SkillUid) -> Option<&dyn Skill> {
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

const fn cost_units(cost: u8) -> u32 {
    cost as u32 * COST_UNIT
}

fn cooldown_slot(owner: &dyn Character, skill: &dyn Skill) -> Option<usize> {
    owner
        .skills()
        .iter()
        .position(|owned| owned.uid() == skill.uid())
}

fn play_ex_skill(
    state: &mut State,
    skill: &dyn Skill,
    caster: &dyn Character,
    min_input_interval: u16,
) -> Result<(), Error> {
    if state.frames() < state.next_input_frame {
        return Err(Error::InvalidArgument("too soon after the previous input"));
    }
    state.next_input_frame = state.frames().saturating_add(min_input_interval);

    let slot = cooldown_slot(caster, skill).ok_or(Error::NotFound)?;
    let data = state.search_uid_mut(caster.uid()).ok_or(Error::NotFound)?;
    let cooldown = data.cooldowns_mut().get_mut(slot).ok_or(Error::NotFound)?;

    if *cooldown != 0 {
        return Err(Error::InvalidArgument("skill is on cooldown"));
    }
    *cooldown = skill.cooldown();

    state.cost = state
        .cost
        .checked_sub(cost_units(skill.cost()))
        .ok_or(Error::InvalidArgument("not enough cost"))?;

    if state.cycle_skill(skill.uid()) {
        Ok(())
    } else {
        Err(Error::InvalidArgument("skill is not in the hand"))
    }
}

/// `None` means the skill never becomes usable because the cost it lacks does not regenerate.
fn ticks_until_ready(
    cooldown: u16,
    skill_cost: u8,
    cost: u32,
    cost_per_second: u32,
) -> Option<u16> {
    let deficit = cost_units(skill_cost).saturating_sub(cost);

    if deficit == 0 {
        return Some(cooldown);
    }
    if cost_per_second == 0 {
        return None;
    }

    let wait = deficit.div_ceil(cost_per_second);
    Some(cooldown.max(wait.min(u16::MAX as u32) as u16))
}

fn regenerated_cost(cost: u32, delta_ticks: u16, cost_per_second: u32) -> u32 {
    // COST_UNIT을 틱당 회복량의 분모에 맞췄으므로 틱당 cost_per_second 단위가 정확히 쌓인다.
    let gained = delta_ticks as u64 * cost_per_second as u64;
    (cost as u64 + gained).min(MAX_COST as u64) as u32
}

fn tick_down(data: &mut StateData, delta_ticks: u16, end_tick: u16) {
    for cooldown in data.cooldowns_mut() {
        *cooldown = cooldown.saturating_sub(delta_ticks);
    }

    // 효과 구간이 [적용, 적용 + 지속)이라 남은 틱이 delta와 같으면 이번에 끝난다.
    data.remained_effects_mut()
        .retain(|effect| effect.end_frame > end_tick);
}

fn coordinate_of(state: &State, uid: Uid) -> Position {
    state
        .search_uid(uid)
        .map(|data| data.coordinate())
        .unwrap_or_default()
}

fn resolve_targets(state: &State, skill: &dyn Skill) -> Vec<Uid> {
    let caster_id = skill.owner_uid();
    let caster_coord = coordinate_of(state, caster_id);

    let mut targets = Vec::new();

    for skill_effect in skill.skill_effects() {
        match skill_effect.targets {
            // 캐스터 자신에 대한 효과는 `Skill::apply`의 caster 인자로 처리
            SkillEffectTarget::Oneself { .. } => {}

            SkillEffectTarget::Student { count, .. } => {
                targets.extend(nearest_students(state, caster_id, caster_coord, count));
            }

            SkillEffectTarget::Boss { .. } => targets.push(state.boss().uid()),

            SkillEffectTarget::Land { region, .. } => {
                targets.extend(units_inside(state, region, caster_coord));
            }
        }
    }

    targets
}

fn nearest_students(state: &State, caster_id: Uid, caster_coord: Position, count: u8) -> Vec<Uid> {
    let mut students: Vec<(Position, Uid)> = state
        .students()
        .iter()
        .map(|student| (student.coordinate(), student.uid()))
        .filter(|student| student.1 != caster_id)
        .collect();

    // 유클리드 거리로 정렬
    students.sort_by(|lhs, rhs| {
        euclidean_distance(caster_coord, lhs.0).total_cmp(&euclidean_distance(caster_coord, rhs.0))
    });

    // 캐스터를 뺀 인원이 count보다 적을 수 있으므로 인덱싱 대신 take.
    students
        .iter()
        .take(count.into())
        .map(|student| student.1)
        .collect()
}

fn units_inside(state: &State, region: Region, caster_coord: Position) -> Vec<Uid> {
    std::iter::once(state.boss())
        .chain(state.students())
        .filter(|unit| is_inside(unit.coordinate(), region, caster_coord))
        .map(|unit| unit.uid())
        .collect()
}

// 틱뎀의 첫 틱은 타격 시각에 들어간다. 지속 구간은 효과와 같은 [시작, 시작 + 지속)이다.
fn hit_frames(timing: EffectTiming, first_frame: u16) -> Vec<u16> {
    match timing {
        EffectTiming::Persistent {
            interval_frames,
            duration_frames,
        } if interval_frames != 0 => (0..duration_frames)
            .step_by(interval_frames as usize)
            .map(|offset| first_frame.saturating_add(offset))
            .collect(),
        _ => vec![first_frame],
    }
}

fn boss_hit_coef(
    skill_effect: &SkillEffect,
    boss_coord: Position,
    caster_coord: Position,
) -> Option<(u16, u16)> {
    match skill_effect.targets {
        SkillEffectTarget::Boss {
            kind: Effect::Damage { coef_num, coef_den },
        } => Some((coef_num, coef_den)),
        SkillEffectTarget::Land {
            kind: Effect::Damage { coef_num, coef_den },
            region,
        } if is_inside(boss_coord, region, caster_coord) => Some((coef_num, coef_den)),
        _ => None,
    }
}
