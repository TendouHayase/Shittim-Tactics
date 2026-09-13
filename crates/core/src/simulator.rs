use crate::{
    actions::ActionContext,
    character::Character,
    skill::{Skill, SkillEffectTarget},
    state::State,
    uid::Uid,
    utils::{Position, euclidean_distance, is_inside},
};

pub trait Simulator {
    fn initial_state(&self) -> State;

    fn legal_actions(&self, state: &State) -> Vec<ActionContext<'_>>;

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

    fn apply(&self, state: State, action: &ActionContext) -> State;

    /// Advances `state` by `delta_ticks`.
    fn advance(&self, state: &State, delta_ticks: u16) -> Result<State, error::Error>;

    /// Ticks from `state` until the next point where anyone can act.
    fn next_event_frames(&self, state: &State) -> u16;

    /// Whether `ticks` is past the time limit.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// The skill at a given skill offset.
    fn lookup_skill(&self, index: usize) -> Option<&dyn Skill>;

    /// The character with this `id`, if there is one.
    fn character_by_uid(&self, id: Uid) -> Option<&dyn Character>;
}
