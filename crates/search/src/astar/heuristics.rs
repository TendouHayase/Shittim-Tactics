use core::{
    actions::ActionContext, agent::Agent, character::CharacterOps, simulator::Simulator,
    skill::SkillMeta, state::Stateful,
};

/// The default agent for A\*.
///
/// `value` is a lower bound on the frames left before the boss reaches the damage threshold, so
/// it stays admissible; `policy` adds no preference of its own and hands back every legal action.
pub struct Heuristic;

impl<'a, S: Stateful<'a>> Agent<'a, S> for Heuristic {
    type Value = u64;

    fn policy(&self, sim: &impl Simulator<'a, S>, state: &S) -> Vec<(ActionContext<'a>, f64)> {
        let actions = sim.legal_actions(state);
        let prior = 1.0 / actions.len() as f64;

        actions.into_iter().map(|action| (action, prior)).collect()
    }

    fn value(&self, sim: &impl Simulator<'a, S>, state: &S) -> Self::Value {
        let boss = state.boss();

        let guard = boss
            .accumulated_damage_cache
            .get_or_compute(&boss.damage_list());
        let Some(dealt) = guard.as_ref() else {
            return 0;
        };

        // 누적 데미지의 최댓값을 빼야 남은 체력이 최소가 되고, 그래야 남은 프레임을
        // 과대평가하지 않는다. 과대평가하면 A*의 최적성이 조용히 깨진다.
        let remain_hp = boss.character.stats().hp.saturating_sub(dealt.max);
        if remain_hp == 0 {
            return 0;
        }

        let max_damage = sim.damage_map().values().max().copied().unwrap_or_default();

        let mut all_ex_damage = 0u64;
        let mut max_ex_dps = 0u64;
        let mut max_ex_frames = 0u16;

        for student in state.students() {
            let damage = student.damage_with_effects().unwrap_or_default();
            all_ex_damage += damage.crit.max;

            // 0번이 EX 스킬이라는 전제. 스킬 목록이 빈 학생은 건너뛴다.
            let Some(frames) = student.character.skill_list().first().map(|s| s.duration()) else {
                continue;
            };
            if frames == 0 {
                continue;
            }

            let dps = damage.crit.max / frames as u64;
            if dps > max_ex_dps {
                max_ex_dps = dps;
                max_ex_frames = frames;
            }
        }

        // 세 하한 중 가장 큰 것이 가장 촘촘하다. 분모가 0인 항은 하한을 못 주므로 뺀다.
        let mut result = 0u64;

        if max_damage.normal.max > 0 {
            result = result.max(remain_hp / max_damage.normal.max);
        }
        if all_ex_damage > 0 {
            result = result.max(remain_hp / all_ex_damage);
        }
        if max_ex_dps > 0 {
            result = result.max(remain_hp.saturating_mul(max_ex_frames as u64) / max_ex_dps);
        }

        result
    }
}
