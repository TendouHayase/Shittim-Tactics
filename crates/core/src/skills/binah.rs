//! 패턴 수치는 전부 `data/bosses/binah.json`에서 옴. 스킬은 난이도도 json도 모르고, 생성될
//! 때 받은 [`params`] 값만 봄.
use crate::create_boss_skill;
use crate::{
    boss::Boss,
    character::{Character, CharacterOps},
    effect::EffectTiming,
    skill::{EffectKind, SkillEffect, SkillEffectTarget, SkillOps, SkillType},
    stat::StatKind,
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;
/// json의 `skills` 객체를 받는 원본 구조체(`Raw*`)와, 난이도 하나를 골라낸 결과(`*Params`).
///
/// 최상위 `struct`로 두면 xtask가 스킬 구조체로 오인해 `Skill` enum에 넣음. 모듈 안에 둘 것.
pub mod params {
    use crate::difficulty::{ByDifficulty, Difficulty};
    use crate::locale::LocalizedName;
    use crate::skill::Region;
    use serde::Deserialize;
    /// 계수는 전부 백분율이라 분모가 고정.
    pub const PERCENT_DEN: u16 = 100;
    /// 비나전 온필드 최대 인원. 실제로 몇 명이 서는지는 런타임에야 알 수 있으므로 이건
    /// `skill_effects`가 효과를 선언할 때 쓰는 상한일 뿐이고, `apply`는 받은 대상을 그대로 씀.
    /// `core::utils::MAX_STUDENT_COUNT`는 편성 최대(10)라 여기 쓰면 안 됨.
    pub const ON_FIELD_COUNT: u8 = 4;
    /// json `skills`의 키는 보스 이름 접두사를 뺀 스킬 구조체 이름.
    #[derive(Debug, Deserialize)]
    pub struct RawSkills {
        #[serde(rename = "AtsilutsLight")]
        pub atsiluts_light: RawAtsilutsLight,
        #[serde(rename = "FiresofSeverity")]
        pub fires_of_severity: RawFiresOfSeverity,
        #[serde(rename = "PurifyingStorm")]
        pub purifying_storm: RawPurifyingStorm,
    }
    #[derive(Debug, Clone, Copy)]
    pub struct AtsilutsLightParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub instant_percent: u16,
        pub dot_percent: u16,
        pub dot_interval: u16,
        pub dot_duration: u16,
        /// 빛이 덮는 세로 직사각형. 보스 기준 상대 좌표.
        pub region: Region,
    }
    #[derive(Debug, Deserialize)]
    pub struct RawAtsilutsLight {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        instant_percent: ByDifficulty<u16>,
        dot_percent: ByDifficulty<u16>,
        dot_interval: ByDifficulty<u16>,
        dot_duration: ByDifficulty<u16>,
        region: ByDifficulty<Region>,
    }
    impl RawAtsilutsLight {
        pub fn pick(&self, difficulty: Difficulty) -> AtsilutsLightParams {
            AtsilutsLightParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                instant_percent: self.instant_percent[difficulty],
                dot_percent: self.dot_percent[difficulty],
                dot_interval: self.dot_interval[difficulty],
                dot_duration: self.dot_duration[difficulty],
                region: self.region[difficulty],
            }
        }
    }
    /// 한 번에 두 방이 나감. 모든 적에게 한 대, 가까운 스트라이커 4명에게 추가로 한 대씩.
    #[derive(Debug, Clone, Copy)]
    pub struct FiresOfSeverityParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub all_percent: u16,
        /// 비나에게 가까운 순서대로 물림. 인원이 고정이라 `(계수, 인원)`이 아니라 순서 배열.
        pub nearest_percents: [u16; 4],
    }
    #[derive(Debug, Deserialize)]
    pub struct RawFiresOfSeverity {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        all_percent: ByDifficulty<u16>,
        nearest_percents: ByDifficulty<[u16; 4]>,
    }
    impl RawFiresOfSeverity {
        pub fn pick(&self, difficulty: Difficulty) -> FiresOfSeverityParams {
            FiresOfSeverityParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                all_percent: self.all_percent[difficulty],
                nearest_percents: self.nearest_percents[difficulty],
            }
        }
    }
    #[derive(Debug, Clone, Copy)]
    pub struct PurifyingStormParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub percent: u16,
        pub def_down_scale: u16,
        pub def_down_duration: u16,
        pub count: u8,
    }
    #[derive(Debug, Deserialize)]
    pub struct RawPurifyingStorm {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        percent: ByDifficulty<u16>,
        def_down_scale: ByDifficulty<u16>,
        def_down_duration: ByDifficulty<u16>,
        count: ByDifficulty<u8>,
    }
    impl RawPurifyingStorm {
        pub fn pick(&self, difficulty: Difficulty) -> PurifyingStormParams {
            PurifyingStormParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                percent: self.percent[difficulty],
                def_down_scale: self.def_down_scale[difficulty],
                def_down_duration: self.def_down_duration[difficulty],
                count: self.count[difficulty],
            }
        }
    }
}
fn damage_effect(percent: u16) -> EffectKind {
    EffectKind::Damage {
        coef_num: percent,
        coef_den: params::PERCENT_DEN,
    }
}
/// `percents`를 대상에 앞에서부터 하나씩 물림. 대상이 모자라면 남은 값은 버려지고, 값이
/// 모자라면 남은 대상은 맞지 않음.
///
/// 순서가 곧 대상 선정이므로 거리순 패턴은 `targets`가 이미 정렬되어 있다고 보고 씀.
fn append_damage(
    caster: &StateData<'_>,
    targets: &mut [&mut StateData<'_>],
    percents: impl IntoIterator<Item = u16>,
    ticks: u16,
) {
    let Some(damage) = caster.damage_with_effects() else {
        return;
    };
    for (target, percent) in targets.iter_mut().zip(percents) {
        target
            .accumulated_damage_cache
            .append(&(damage * percent as u64 / params::PERCENT_DEN as u64));
        target.accumulated_damage.push(AccumulatedDamage {
            ticks,
            damage: target.damage_map.get(&target.effects).copied(),
        });
    }
}
create_boss_skill!(
    BinahAtsilutsLight, params : params::AtsilutsLightParams, SkillType::Ex, 0, { fn
    skill_effects(& self) -> Vec < SkillEffect > { let params = self.params;
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Land { kind : damage_effect(params.instant_percent), region :
    params.region, }], }, SkillEffect { id : self.id, timing : EffectTiming::Persistent {
    interval_frames : params.dot_interval, duration_frames : params.dot_duration, },
    targets : vec![SkillEffectTarget::Land { kind : damage_effect(params.dot_percent),
    region : params.region, }], },] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, _caster :
    &'c mut StateData <'a >, _targets : &'b mut [&'c mut StateData <'a >],) { todo!() } }
);
create_boss_skill!(
    BinahFiresofSeverity, params : params::FiresOfSeverityParams, SkillType::Ex, 1, { fn
    skill_effects(& self) -> Vec < SkillEffect > { let params = self.params;
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Student { kind : damage_effect(params.all_percent), count :
    params::ON_FIELD_COUNT, }], }, SkillEffect { id : self.id, timing :
    EffectTiming::Instant, targets : params.nearest_percents.iter().map(|& percent |
    SkillEffectTarget::Student { kind : damage_effect(percent), count : 1, }).collect(),
    },] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >,
    targets : &'b mut [&'c mut StateData <'a >],) { let params = self.params; let ticks =
    self.duration(); append_damage(caster, targets, std::iter::repeat(params
    .all_percent), ticks); append_damage(caster, targets, params.nearest_percents,
    ticks); } }
);
create_boss_skill!(
    BinahPurifyingStorm, params : params::PurifyingStormParams, SkillType::Ex, 2, { fn
    skill_effects(& self) -> Vec < SkillEffect > { let params = self.params;
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Student { kind : EffectKind::Debuff { ty : StatKind::Def,
    duration : params.def_down_duration, scale : params.def_down_scale, amount : 0, },
    count : params.count, }], }, SkillEffect { id : self.id, timing :
    EffectTiming::Instant, targets : vec![SkillEffectTarget::Student { kind :
    damage_effect(params.percent), count : params.count, }], },] } fn apply <'a : 'b, 'b,
    'c : 'b > (& self, caster : &'c mut StateData <'a >, targets : &'b mut [&'c mut
    StateData <'a >],) { let params = self.params; append_damage(caster, targets,
    std::iter::repeat_n(params.percent, params.count as usize), self.duration(),); } }
);
