//! 패턴 수치는 [`params`] 한 곳에 모아뒀다. `data/bosses/perorodzilla.json` 파서가 붙으면
//! [`params::Params::of`]와 프레임 상수만 갈아끼우면 된다.
use crate::create_boss_skill;
use crate::states::PerorodzillaState;
use crate::{
    boss::Boss, character::Character, damage::Damage, difficulty::Difficulty,
    skill::{
        BuffType, DebuffType, EffectKind, EffectTiming, Region, Skill, SkillEffect,
        SkillEffectTarget, SkillOps, SkillType,
    },
    state::{AccumulatedDamage, State, StateData, Stateful},
    types::AttackType, utils::{MAX_STUDENT_COUNT, is_inside},
};
use std::ptr::NonNull;
use params::Params;
/// json에 없는 패턴 수치. 아직 데이터가 없는 항목은 `None`/`0`이고, 그 값을 쓰는 효과는
/// 조용히 빠진다.
///
/// # Warning
///
/// 최상위 `struct`로 두면 xtask가 스킬 구조체로 오인해 `Skill` enum에 넣는다. 모듈 안에 둘 것.
mod params {
    use crate::difficulty::Difficulty;
    use crate::skill::Region;
    use crate::utils::{MAX_STUDENT_COUNT, time_to_ticks};
    /// 계수는 전부 백분율이라 분모가 고정.
    pub const PERCENT_DEN: u16 = 100;
    /// `0`은 미측정.
    ///
    /// 3.666초. `time_to_ticks(3666, 1000)`은 `3666 * 30`이 u16을 넘어 터진다.
    pub const WHITE_HOT_HEAT_VISION_FRAMES: u16 = time_to_ticks(1833, 500);
    pub const AQUA_BALL_FRAMES: u16 = 0;
    pub const SUMMON_MINION_FRAMES: u16 = 0;
    pub const ABSORB_MINION_FRAMES: u16 = 0;
    pub const HYPER_SPIRAL_GLARE_BEAM_FRAMES: u16 = 0;
    /// 매크로 인자는 `self`를 볼 수 없어서 상수로 뺀다.
    pub const DOT_DURATION: u16 = time_to_ticks(10, 1);
    #[derive(Debug, Clone, Copy)]
    pub struct Params {
        pub def_down_amount: u32,
        pub def_down_duration: u16,
        pub dot_interval: u16,
        pub dot_duration: u16,
        pub heat_vision_percent: u16,
        pub chain_count: u8,
        /// 대상이 더 많으면 마지막 값을 반복한다.
        pub chain_percents: [u16; 2],
        pub blast_percent: u16,
        pub blast_region: Option<Region>,
        pub blast_atk_down_scale: u16,
        pub blast_atk_down_duration: u16,
        pub aqua_ball_percent: u16,
        pub aqua_ball_region: Option<Region>,
        pub aqua_ball_def_down: bool,
        pub hyper_spiral_percent: u16,
        pub big_minion_count: u8,
        pub shiny_minion_count: u8,
        pub shiny_blast_damage: u64,
        pub shiny_blast_region: Option<Region>,
        pub groggy_denominator: u8,
        pub small_minion_count: u8,
        pub knockback_on_groggy: bool,
        pub atg_gain_percent: u16,
        pub mystic_up_percent: u16,
    }
    impl Params {
        pub fn of(difficulty: Difficulty) -> Self {
            let mut params = Self {
                def_down_amount: 9_999,
                def_down_duration: time_to_ticks(60, 1),
                dot_interval: time_to_ticks(1, 1),
                dot_duration: DOT_DURATION,
                heat_vision_percent: 90,
                chain_count: 0,
                chain_percents: [0, 0],
                blast_percent: 0,
                blast_region: None,
                blast_atk_down_scale: 0,
                blast_atk_down_duration: 0,
                aqua_ball_percent: 400,
                aqua_ball_region: None,
                aqua_ball_def_down: false,
                hyper_spiral_percent: 300,
                big_minion_count: 7,
                shiny_minion_count: 0,
                shiny_blast_damage: 200_000,
                shiny_blast_region: None,
                groggy_denominator: 7,
                small_minion_count: 5,
                knockback_on_groggy: false,
                atg_gain_percent: 50,
                mystic_up_percent: 0,
            };
            if matches!(
                difficulty, Difficulty::Insane | Difficulty::Torment |
                Difficulty::Lunatic
            ) {
                params.blast_percent = 250;
                params.shiny_minion_count = 1;
                params.groggy_denominator = 10;
            }
            match difficulty {
                Difficulty::Torment => {
                    params.heat_vision_percent = 100;
                    params.chain_count = 2;
                    params.chain_percents = [20, 10];
                    params.blast_atk_down_scale = 50;
                    params.blast_atk_down_duration = time_to_ticks(40, 1);
                    params.groggy_denominator = 12;
                    params.mystic_up_percent = 50;
                }
                Difficulty::Lunatic => {
                    params.heat_vision_percent = 100;
                    params.chain_count = (MAX_STUDENT_COUNT - 1) as u8;
                    params.chain_percents = [40, 40];
                    params.blast_atk_down_scale = 50;
                    params.blast_atk_down_duration = time_to_ticks(40, 1);
                    params.aqua_ball_def_down = true;
                    params.groggy_denominator = 12;
                    params.knockback_on_groggy = true;
                    params.mystic_up_percent = 200;
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
fn difficulty_of(skill: &Skill) -> Difficulty {
    match skill.owner() {
        Character::Boss(boss) => boss.stats.difficulty,
        Character::Student(_) => {
            unreachable!("Perorodzilla skills always belong to a boss")
        }
    }
}
fn append_damage_over_time(
    caster: &StateData<'_>,
    target: &mut StateData<'_>,
    percent: u16,
    interval: u16,
    duration: u16,
) {
    let Some(damage) = caster.damage_with_effects() else {
        return;
    };
    let mut ticks = interval;
    while ticks <= duration {
        target
            .accumulated_damage_cache
            .append(&(damage * percent as u64 / params::PERCENT_DEN as u64));
        target
            .accumulated_damage
            .push(AccumulatedDamage {
                ticks,
                damage: target.damage_map.get(&target.effects).copied(),
            });
        ticks += interval;
    }
}
fn append_damage(
    caster: &StateData<'_>,
    target: &mut StateData<'_>,
    percent: u16,
    ticks: u16,
) {
    let Some(damage) = caster.damage_with_effects() else {
        return;
    };
    target
        .accumulated_damage_cache
        .append(&(damage * percent as u64 / params::PERCENT_DEN as u64));
    target
        .accumulated_damage
        .push(AccumulatedDamage {
            ticks,
            damage: target.damage_map.get(&target.effects).copied(),
        });
}
fn summon_minion_wave(boss: &mut StateData<'_>, params: Params) {
    let record_start = boss.accumulated_damage.len();
    let pero = boss.extra_as_mut::<PerorodzillaState>();
    pero.big_minions = params.big_minion_count;
    pero.shiny_minions = params.shiny_minion_count;
    pero.knocked_down = 0;
    pero.minion_damage = 0;
    pero.damage_record_start = record_start;
}
/// 미니온이 받은 데미지 100%가 보스에게 들어가므로 보스 데미지 기록의 증가분을 미니온이
/// 받은 몫으로 본다. 웨이브 중에는 미니온이 우선 타깃이라 대부분의 데미지가 미니온을
/// 거친다고 가정한 근사다.
fn damage_since_wave_start(boss: &StateData<'_>) -> u64 {
    let record_start = boss
        .extra_as::<PerorodzillaState>()
        .damage_record_start
        .min(boss.accumulated_damage.len());
    boss.accumulated_damage[record_start..]
        .iter()
        .filter_map(|acc| acc.damage)
        .map(|damage| damage.expected_value())
        .sum()
}
/// 미니온은 한 마리씩 우선 타깃이 되므로 웨이브 전체가 받은 데미지를 체력의 50%로 나눈
/// 몫을 넘어진 마리 수로 본다. 체력을 모르면(`big_minion_hp == 0`) 판정할 수 없다.
fn knockdown_count(boss: &StateData<'_>) -> u8 {
    let pero = boss.extra_as::<PerorodzillaState>();
    let threshold = pero.big_minion_hp / 2;
    if threshold == 0 {
        return 0;
    }
    ((pero.minion_damage / threshold) as u8).min(pero.big_minions)
}
/// 폭발 범위 안의 모든 대상에게 데미지가 들어가고 보스는 그 합을 받는다. 반환값은 큰
/// 미니온이 받은 몫으로, 다시 넘어짐 판정에 들어간다.
///
/// 큰 미니온은 개별 좌표가 없어 아직 서 있는 미니온 전부가 범위 안이라고 본다.
fn apply_shiny_minion_blast(
    boss: &mut StateData<'_>,
    students: &mut [&mut StateData<'_>],
    params: Params,
    region: Region,
    ticks: u16,
) -> u64 {
    let shiny_count = params.shiny_minion_count as u64;
    let unit = params.shiny_blast_damage;
    let blast = Damage::new(unit, unit, unit, unit, 0, 1, 0);
    let origin = boss.coordinate;
    let mut student_share = 0u64;
    for student in students.iter_mut() {
        if !is_inside(student.coordinate, region, origin) {
            continue;
        }
        for _ in 0..shiny_count {
            student.accumulated_damage_cache.append(&blast);
            student
                .accumulated_damage
                .push(AccumulatedDamage {
                    ticks,
                    damage: Some(blast),
                });
        }
        student_share += unit * shiny_count;
    }
    let pero = boss.extra_as::<PerorodzillaState>();
    let standing = pero.big_minions.saturating_sub(pero.knocked_down) as u64;
    let minion_share = unit * shiny_count * standing;
    let total = minion_share + student_share;
    if total > 0 {
        let absorbed = Damage::new(total, total, total, total, 0, 1, 0);
        boss.accumulated_damage_cache.append(&absorbed);
        boss.accumulated_damage
            .push(AccumulatedDamage {
                ticks,
                damage: Some(absorbed),
            });
    }
    minion_share
}
/// 그로기 게이지가 다 찼으면 `true`.
fn absorb_minion_wave(
    boss: &mut StateData<'_>,
    students: &mut [&mut StateData<'_>],
    params: Params,
) -> bool {
    let dealt = damage_since_wave_start(boss);
    boss.extra_as_mut::<PerorodzillaState>().minion_damage = dealt;
    let mut knocked = knockdown_count(boss);
    if knocked > 0 && params.shiny_minion_count > 0
        && let Some(region) = params.shiny_blast_region
    {
        boss.extra_as_mut::<PerorodzillaState>().knocked_down = knocked;
        let minion_share = apply_shiny_minion_blast(boss, students, params, region, 1);
        boss.extra_as_mut::<PerorodzillaState>().minion_damage += minion_share;
        knocked = knockdown_count(boss);
    }
    let pero = boss.extra_as_mut::<PerorodzillaState>();
    pero.knocked_down = knocked;
    pero.groggy_numerator += knocked;
    pero.big_minions = 0;
    pero.shiny_minions = 0;
    pero.atg_percent = (pero.atg_percent + params.atg_gain_percent).min(100);
    if pero.groggy_numerator >= params.groggy_denominator {
        pero.groggy_numerator = 0;
        pero.small_minions = params.small_minion_count;
        true
    } else {
        false
    }
}
/// json을 읽는 쪽에서 전투 시작 시 한 번 호출해야 한다. 채우지 않으면 넘어짐 판정이
/// 비활성화된다.
pub fn init_big_minion_hp(boss: &mut StateData<'_>, hp: u64) {
    boss.extra_as_mut::<PerorodzillaState>().big_minion_hp = hp;
}
create_boss_skill!(
    PerorodzillaWhiteHotHeatVision, 0, params::DOT_DURATION,
    params::WHITE_HOT_HEAT_VISION_FRAMES, SkillType::Ex, 0, params : params::Params, { fn
    skill_effects(& self) -> Vec < SkillEffect > { let params = self.params; let
    dot_timing = EffectTiming::Persistent { interval_frames : params.dot_interval,
    duration_frames : params.dot_duration, }; let mut effects = vec![SkillEffect { id :
    self.id, timing : EffectTiming::Instant, targets : vec![SkillEffectTarget::Student {
    kind : EffectKind::Debuff { ty : DebuffType::Def, duration : params
    .def_down_duration, scale : 0, amount : params.def_down_amount, }, count : 1, }], },
    SkillEffect { id : self.id, timing : dot_timing, targets :
    vec![SkillEffectTarget::Student { kind : damage_effect(params.heat_vision_percent),
    count : 1, }], },]; if params.chain_count > 0 { let [first, rest] = params
    .chain_percents; let mut chain = vec![SkillEffectTarget::Student { kind :
    damage_effect(first), count : 1, }]; if params.chain_count > 1 { chain
    .push(SkillEffectTarget::Student { kind : damage_effect(rest), count : params
    .chain_count - 1, }); } effects.push(SkillEffect { id : self.id, timing : dot_timing,
    targets : chain, }); } if params.blast_percent > 0 && let Some(region) = params
    .blast_region { let mut targets = vec![SkillEffectTarget::Land { kind :
    damage_effect(params.blast_percent), region, }]; if params.blast_atk_down_scale > 0 {
    targets.push(SkillEffectTarget::Land { kind : EffectKind::Debuff { ty :
    DebuffType::Atk, duration : params.blast_atk_down_duration, scale : params
    .blast_atk_down_scale, amount : 0, }, region, }); } effects.push(SkillEffect { id :
    self.id, timing : EffectTiming::Instant, targets, }); } effects } fn apply <'a : 'b,
    'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >, targets : &'b mut [&'c mut
    StateData <'a >],) { let params = self.params; for (i, target) in targets.iter_mut()
    .enumerate() { let percent = match i { 0 => params.heat_vision_percent, _ if (i as
    u8) <= params.chain_count => { let last = params.chain_percents.len() - 1; params
    .chain_percents[(i - 1).min(last)] } _ => continue, };
    append_damage_over_time(caster, target, percent, params.dot_interval, params
    .dot_duration,); } } }
);
create_boss_skill!(
    PerorodzillaAquaBall, 0, 0, params::AQUA_BALL_FRAMES, SkillType::Ex, 1, params :
    params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let params = self
    .params; let Some(region) = params.aqua_ball_region else { return vec![]; }; let mut
    targets = vec![SkillEffectTarget::Land { kind : damage_effect(params
    .aqua_ball_percent), region, }]; if params.aqua_ball_def_down { targets
    .push(SkillEffectTarget::Land { kind : EffectKind::Debuff { ty : DebuffType::Def,
    duration : params.def_down_duration, scale : 0, amount : params.def_down_amount, },
    region, }); } vec![SkillEffect { id : self.id, timing : EffectTiming::Instant,
    targets, }] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData
    <'a >, targets : &'b mut [&'c mut StateData <'a >],) { let percent = self.params
    .aqua_ball_percent; for target in targets.iter_mut() { append_damage(caster, target,
    percent, self.duration()); } } }
);
create_boss_skill!(
    PerorodzillaSummonMinion, 0, 0, params::SUMMON_MINION_FRAMES, SkillType::Ex, 2,
    params : params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > {
    vec![SkillEffect { id : self.id, timing : EffectTiming::Instant, targets :
    vec![SkillEffectTarget::Oneself { kind : EffectKind::new_other(Self::other_apply),
    }], }] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >,
    _targets : &'b mut [&'c mut StateData <'a >],) { summon_minion_wave(caster, self
    .params); } }
);
impl PerorodzillaSummonMinion {
    pub fn other_apply<'a>(skill: &Skill, mut state: State<'a>) -> State<'a> {
        let params = Params::of(difficulty_of(skill));
        summon_minion_wave(state.boss_mut(), params);
        state
    }
}
create_boss_skill!(
    PerorodzillaAbsorbMinion, 0, 0, params::ABSORB_MINION_FRAMES, SkillType::Ex, 3,
    params : params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let
    params = self.params; let mut targets = vec![SkillEffectTarget::Oneself { kind :
    EffectKind::new_other(Self::other_apply), }]; if params.knockback_on_groggy { targets
    .push(SkillEffectTarget::Student { kind : EffectKind::Move, count : MAX_STUDENT_COUNT
    as u8, }); } vec![SkillEffect { id : self.id, timing : EffectTiming::Instant,
    targets, }] } fn apply <'a : 'b, 'b, 'c : 'b > (& self, caster : &'c mut StateData
    <'a >, targets : &'b mut [&'c mut StateData <'a >],) { let params = self.params; let
    is_groggy = absorb_minion_wave(caster, targets, params); let _ = is_groggy && params
    .knockback_on_groggy; } }
);
impl PerorodzillaAbsorbMinion {
    pub fn other_apply<'a>(skill: &Skill, mut state: State<'a>) -> State<'a> {
        let params = Params::of(difficulty_of(skill));
        let (boss, students) = state.split_mut();
        let mut students: Vec<&mut StateData<'_>> = students.iter_mut().collect();
        absorb_minion_wave(boss, &mut students, params);
        state
    }
}
create_boss_skill!(
    PerorodzillaHyperSpiralGlareBeam, 0, 0, params::HYPER_SPIRAL_GLARE_BEAM_FRAMES,
    SkillType::Ex, 4, params : params::Params, { fn skill_effects(& self) -> Vec <
    SkillEffect > { vec![SkillEffect { id : self.id, timing : EffectTiming::Instant,
    targets : vec![SkillEffectTarget::Student { kind : damage_effect(self.params
    .hyper_spiral_percent), count : MAX_STUDENT_COUNT as u8, }], }] } fn apply <'a : 'b,
    'b, 'c : 'b > (& self, caster : &'c mut StateData <'a >, targets : &'b mut [&'c mut
    StateData <'a >],) { if caster.extra_as::< PerorodzillaState > ().atg_percent < 100 {
    return; } let percent = self.params.hyper_spiral_percent; for target in targets
    .iter_mut() { append_damage(caster, target, percent, self.duration()); } caster
    .extra_as_mut::< PerorodzillaState > ().atg_percent = 0; } }
);
create_boss_skill!(
    PerorodzillaBurningPerorodzilla, 0, 0, 0, SkillType::Passive, 5, params :
    params::Params, { fn skill_effects(& self) -> Vec < SkillEffect > { let scale = self
    .params.mystic_up_percent; if scale == 0 { return vec![]; } vec![SkillEffect { id :
    self.id, timing : EffectTiming::Instant, targets : vec![SkillEffectTarget::Oneself {
    kind : EffectKind::Buff { ty : BuffType::Effectiveness(AttackType::Mystic), duration
    : u16::MAX, scale, amount : 0, }, }], }] } fn apply <'a : 'b, 'b, 'c : 'b > (& self,
    _caster : &'c mut StateData <'a >, _targets : &'b mut [&'c mut StateData <'a >],) {}
    }
);
