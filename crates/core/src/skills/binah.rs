//! 패턴 수치는 [`params`] 한 곳에 모아뒀다. `data/bosses/binah.json` 파서가 붙으면
//! [`params::Params::of`]와 프레임 상수만 갈아끼우면 된다.
use crate::create_boss_skill;
use crate::{
    boss::Boss,
    character::Character,
    skill::{
        DebuffType, EffectKind, EffectTiming, SkillEffect, SkillEffectTarget, SkillOps, SkillType,
    },
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;
/// json에 없는 패턴 수치. 아직 데이터가 없는 항목은 `0`이고, 그 값을 쓰는 효과는 조용히
/// 빠진다.
///
/// # Warning
///
/// 최상위 `struct`로 두면 xtask가 스킬 구조체로 오인해 `Skill` enum에 넣는다. 모듈 안에 둘 것.
mod params {
    use crate::difficulty::Difficulty;
    use crate::skill::Region;
    use crate::utils::time_to_ticks;
    /// 계수는 전부 백분율이라 분모가 고정.
    pub const PERCENT_DEN: u16 = 100;
    /// `0`은 미측정.
    pub const ATSILUTS_LIGHT_FRAMES: u16 = 0;
    pub const FIRES_OF_SEVERITY_1_FRAMES: u16 = 0;
    pub const FIRES_OF_SEVERITY_2_FRAMES: u16 = 0;
    pub const PURIFYING_STORM_FRAMES: u16 = 0;
    /// 매크로 인자는 `self`를 볼 수 없어서 상수로 뺀다.
    pub const PURIFYING_STORM_DURATION: u16 = 30;
    /// 아트질루트의 빛이 덮는 세로 직사각형. 보스 기준 상대 좌표이고 난이도와 무관.
    ///
    /// `Position`의 필드가 `OrderedFloat`이고 `bosses`는 `ordered_float`에 의존하지 않아
    /// `const`로는 못 만든다.
    pub fn light_region() -> Region {
        Region::Polygon {
            vertex: [
                (-150, 2200).into(),
                (150, 2200).into(),
                (150, 0).into(),
                (-150, 0).into(),
            ],
            count: 4,
        }
    }
    /// `(계수 백분율, 인원)`. 앞에서부터 순서대로 대상에게 배분된다.
    pub type Split = (u16, u8);
    #[derive(Debug, Clone, Copy)]
    pub struct Params {
        pub light_instant_percent: u16,
        pub light_dot_percent: u16,
        pub light_dot_interval: u16,
        pub light_dot_duration: u16,
        pub severity_1_split: Split,
        pub severity_2_splits: [Split; 3],
        pub storm_percent: u16,
        pub storm_def_down_scale: u16,
        pub storm_def_down_duration: u16,
        pub storm_count: u8,
    }
    impl Params {
        pub fn of(difficulty: Difficulty) -> Self {
            let mut params = Self {
                light_instant_percent: 120,
                light_dot_percent: 50,
                light_dot_interval: time_to_ticks(3, 1),
                light_dot_duration: time_to_ticks(15, 1),
                severity_1_split: (75, 4),
                severity_2_splits: [(375, 1), (150, 2), (75, 1)],
                storm_percent: 300,
                storm_def_down_scale: 50,
                storm_def_down_duration: 90,
                storm_count: 4,
            };
            if matches!(
                difficulty,
                Difficulty::Insane | Difficulty::Torment | Difficulty::Lunatic
            ) {
                params.severity_1_split = (150, 4);
                params.severity_2_splits = [(750, 1), (300, 2), (150, 1)];
            }
            match difficulty {
                Difficulty::Torment => {
                    params.light_instant_percent = 160;
                }
                Difficulty::Lunatic => {
                    params.light_instant_percent = 200;
                    params.light_dot_percent = 130;
                    params.light_dot_duration = time_to_ticks(120, 1);
                }
                _ => {}
            }
            params
        }
    }
}
fn damage_effect(percent: u16) -> EffectKind {
    EffectKind::Damage {
        coef_num: percent,
        coef_den: params::PERCENT_DEN,
    }
}
/// `splits`를 앞에서부터 인원수만큼 펼쳐 대상에 하나씩 물린다. 대상이 모자라면 남은
/// split은 버려짐.
fn append_split_damage(
    caster: &StateData<'_>,
    targets: &mut [&mut StateData<'_>],
    splits: &[params::Split],
    ticks: u16,
) {
    let Some(damage) = caster.damage_with_effects() else {
        return;
    };
    let percents = splits
        .iter()
        .flat_map(|&(percent, count)| std::iter::repeat_n(percent, count as usize));
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
    BinahAtsilutsLight, 0, 0, params::ATSILUTS_LIGHT_FRAMES, SkillType::Ex, 0, params :
    params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let params = self
    .params; vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Land { kind : damage_effect(params.light_instant_percent),
    region : params::light_region(), }], }, SkillEffect { id : self.id, timing :
    EffectTiming::Persistent { interval_frames : params.light_dot_interval,
    duration_frames : params.light_dot_duration, }, targets :
    vec![SkillEffectTarget::Land { kind : damage_effect(params.light_dot_percent), region
    : params::light_region(), }], },] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, caster
    : &'c mut StateData <'a >, targets : &'b mut [&'c mut StateData <'a >],) { todo!() }
    }
);
create_boss_skill!(
    BinahFiresofSeverity1, 0, 0, params::FIRES_OF_SEVERITY_1_FRAMES, SkillType::Ex, 1,
    params : params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let
    (percent, count) = self.params.severity_1_split; vec![SkillEffect { id : self.id,
    timing : EffectTiming::Instant, targets : vec![SkillEffectTarget::Student { kind :
    damage_effect(percent), count, }], }] } fn apply <'a : 'b, 'b, 'c : 'b > (& self,
    caster : &'c mut StateData <'a >, targets : &'b mut [&'c mut StateData <'a >],) { let
    split = self.params.severity_1_split; append_split_damage(caster, targets, & [split],
    self.duration()); } }
);
create_boss_skill!(
    BinahFireofSeverity2, 0, 0, params::FIRES_OF_SEVERITY_2_FRAMES, SkillType::Ex, 2,
    params : params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let
    targets = self.params.severity_2_splits.iter().map(|& (percent, count) |
    SkillEffectTarget::Student { kind : damage_effect(percent), count, }).collect();
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets, }] } fn
    apply <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >, targets :
    &'b mut [&'c mut StateData <'a >],) { let splits = self.params.severity_2_splits;
    append_split_damage(caster, targets, & splits, self.duration()); } }
);
create_boss_skill!(
    BinahPurifyingStorm, 3, params::PURIFYING_STORM_DURATION,
    params::PURIFYING_STORM_FRAMES, SkillType::Ex, 3, params : params::Params, { fn
    skill_effects(& self) -> Vec < SkillEffect > { let params = self.params;
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Student { kind : EffectKind::Debuff { ty : DebuffType::Def,
    duration : params.storm_def_down_duration, scale : params.storm_def_down_scale,
    amount : 0, }, count : params.storm_count, }], }, SkillEffect { id : self.id, timing
    : EffectTiming::Instant, targets : vec![SkillEffectTarget::Student { kind :
    damage_effect(params.storm_percent), count : params.storm_count, }], },] } fn apply
    <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >, targets : &'b mut
    [&'c mut StateData <'a >],) { let params = self.params; let split = (params
    .storm_percent, params.storm_count); append_split_damage(caster, targets, & [split],
    self.duration()); } }
);
