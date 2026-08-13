use std::collections::HashMap;

use crate::{
    actions::ActionContext,
    character::Character,
    damage::{Damage, key::SkillsBitMask},
    skill::Skill,
    state::Stateful,
};

/// Implemented by whatever actually runs a simulation.
///
/// The `'a` in `S<'a>` must be the lifetime of the characters the simulation struct holds.
pub trait Simulator {
    type S<'a>: Stateful<'a>;

    /// 에이전트가 현재 `state`에서 할 수 있는 액션(스킬) 목록을 반환합니다.
    fn legal_actions<'a>(&self, state: &impl Stateful<'a>) -> Vec<&Skill>;

    /// Applies `action` to `state` and returns the resulting state.
    ///
    /// `action.targets` must not contain the caster; effects on the caster go through the
    /// `caster` argument of `Skill::apply`. Two mutable references to the same target cannot
    /// coexist, so a caster listed among the targets would simply be skipped.
    fn apply<'a>(&self, state: &Self::S<'a>, action: &ActionContext) -> Self::S<'a>;

    /// 주어진 `state`를 `delta_ticks`만큼 진행시키고 변화된 `state`를 반환합니다.
    fn advance<'a>(
        &self,
        state: &Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error>;

    /// 현재 `state`에서 다음 행동할 수 있는 지점까지 걸리는 tick을 반환합니다.
    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16;

    /// 키로 데미지를 구하는 해시맵을 반환합니다.
    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage>;

    /// 주어진 틱이 시간제한을 넘겼는지 검사합니다.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// `SkillBitMask`에서 주어진 인덱스에 해당하는 스킬을 반환합니다.
    fn lookup_skill(&self, index: usize) -> Result<&Skill, error::Error>;

    /// 주어진 `id`에 맞는 캐릭터가 존재하면 반환하고 아니면 `None`을 반환합니다.
    fn character_by_id(&self, id: u32) -> Option<Character<'_>>;
}
