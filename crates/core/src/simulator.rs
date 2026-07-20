use std::{collections::HashMap, sync::Arc};

use crate::{
    actions::ActionContext,
    character::Character,
    damage::{Damage, key::SkillsBitMask},
    skill::Skill,
    state::Stateful,
};

/// 실제 시뮬레이션을 수행할 시뮬레이터가 구현해야할 크레이트입니다.
///
/// # 구현 시 주의사항
/// 이 트레이트를 구현하며 `S<'a>`의 'a 라이프타임은 실제 시뮬레이션 구조체가 가진 캐릭터의
/// 라이프타임과 같아야합니다.
pub trait Simulator {
    type S<'a>: Stateful<'a>;

    /// 에이전트가 현재 `state`에서 할 수 있는 액션(스킬) 목록을 반환합니다.
    fn legal_actions<'a>(&self, state: &impl Stateful<'a>) -> Vec<Arc<dyn Skill>>;

    /// 실행한 `action`을 `state`에 적용하여 새로운 `state`를 반환합니다.
    fn apply<'a: 'b, 'b, 'c>(
        &self,
        state: &'b Self::S<'a>,
        action: &'b ActionContext<dyn Skill + 'c>,
    ) -> Self::S<'a>;

    /// 주어진 `state`를 `delta_ticks`만큼 진행시키고 변화된 `state`를 반환합니다.
    fn advance<'a: 'b, 'b>(
        &self,
        state: &'b Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error>;

    /// 현재 `state`에서 다음 행동할 수 있는 지점까지 걸리는 tick을 반환합니다.
    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16;

    /// 키로 데미지를 구하는 해시맵을 반환합니다.
    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage>;

    /// 주어진 틱이 시간제한을 넘겼는지 검사합니다.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// `SkillBitMask`에서 주어진 인덱스에 해당하는 스킬을 반환합니다.
    fn lookup_skill(&self, index: usize) -> Result<Arc<dyn Skill>, error::Error>;

    /// 주어진 `id`에 맞는 캐릭터가 존재하면 반환하고 아니면 `None`을 반환합니다.
    fn character_by_id(&self, id: u32) -> Option<&dyn Character>;
}
