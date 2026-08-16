use std::collections::HashMap;

use crate::{
    actions::ActionContext,
    character::{Character, CharacterOps},
    damage::{Damage, key::SkillsBitMask},
    skill::{Skill, SkillEffectTarget, SkillMeta, SkillOps},
    state::Stateful,
    utils::{Position, euclidean_distance, is_inside},
};

pub trait Simulator<'a, S: Stateful<'a>> {
    fn initial_state(&'a self) -> S;

    /// Actions an agent may take from `state`.
    ///
    /// Only legality belongs here — cooldowns, cost, and who a skill hits. Which of these is
    /// worth taking is the agent's call.
    fn legal_actions(&self, state: &S) -> Vec<ActionContext<'a>>;

    /// Who `skill` hits when cast from `state`, ready to hand to `apply`.
    ///
    /// Target selection follows from the skill's `SkillEffectTarget` list and the positions in
    /// `state`, so it is a rule rather than a search decision and lives here instead of in each
    /// algorithm. An agent that wants a different combination may build its own list, but must
    /// still run it through `normalize_targets`.
    fn resolve_targets(&self, state: &S, skill: &Skill) -> Vec<u32> {
        let caster_id = skill.owner().id();
        let caster_coord = state
            .state_data_by_id(caster_id)
            .map(|data| data.coordinate)
            .unwrap_or_default();

        let mut targets = Vec::new();

        for skill_effect in skill.skill_effects() {
            for target in skill_effect.targets {
                match target {
                    // 캐스터 자신에 대한 효과는 `Skill::apply`의 caster 인자로 처리한다.
                    // 여기 넣어봐야 `apply`가 걸러낸다.
                    SkillEffectTarget::Oneself { .. } => {}

                    SkillEffectTarget::Student { count, .. } => {
                        let mut students: Vec<(Position, u32)> = state
                            .students()
                            .iter()
                            .map(|student| (student.coordinate, student.character.id()))
                            .filter(|student| student.1 != caster_id)
                            .collect();

                        students.sort_by(|lhs, rhs| {
                            euclidean_distance(caster_coord, lhs.0)
                                .total_cmp(&euclidean_distance(caster_coord, rhs.0))
                        });

                        // 캐스터를 뺀 인원이 count보다 적을 수 있으므로 인덱싱 대신 take.
                        targets.extend(students.iter().take(count.into()).map(|s| s.1));
                    }

                    SkillEffectTarget::Boss { .. } => targets.push(state.boss().character.id()),

                    SkillEffectTarget::Land { region, .. } => {
                        if is_inside(state.boss().coordinate, region, caster_coord) {
                            targets.push(state.boss().character.id());
                        }

                        for student in state.students() {
                            if is_inside(student.coordinate, region, caster_coord) {
                                targets.push(student.character.id());
                            }
                        }
                    }
                }
            }
        }

        self.normalize_targets(caster_id, &mut targets);
        targets
    }

    /// Enforces what `apply` requires of `action.targets`: no caster, no duplicates.
    fn normalize_targets(&self, caster_id: u32, targets: &mut Vec<u32>) {
        targets.retain(|id| *id != caster_id);
        targets.sort_unstable();
        targets.dedup();
    }

    /// Applies `action` to `state` and returns the resulting state.
    ///
    /// `action.targets` must not contain the caster; effects on the caster go through the
    /// `caster` argument of `Skill::apply`. Two mutable references to the same target cannot
    /// coexist, so a caster listed among the targets would simply be skipped.
    fn apply(&self, state: &S, action: &ActionContext) -> S;

    /// Advances `state` by `delta_ticks`.
    fn advance(&self, state: &S, delta_ticks: u16) -> Result<S, error::Error>;

    /// Ticks from `state` until the next point where anyone can act.
    fn next_event_frames(&self, state: &S) -> u16;

    /// Damage keyed by which skills are active.
    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage>;

    /// Whether `ticks` is past the time limit.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// The skill at a given `SkillsBitMask` index.
    fn lookup_skill(&self, index: usize) -> Result<&Skill, error::Error>;

    /// The character with this `id`, if there is one.
    fn character_by_id(&self, id: u32) -> Option<Character<'_>>;
}
