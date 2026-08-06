use crate::states::KeiState;
use crate::{
    character::{Character, CharacterOps},
    damage::Damage,
    effect::EffectTiming,
    skill::{EffectKind, Skill, SkillEffect, SkillEffectTarget, SkillOps, SkillType},
    stat::StatKind,
    state::{AccumulatedDamage, RemainedEffects, State, StateData, Stateful},
    student::Student,
    types::AttackType,
    utils::is_inside,
};
use std::{cmp::Reverse, ptr::NonNull};
/// json에 없는 스킬 수치. 파서가 붙으면 각 `new`에 넘길 값만 데이터에서 읽으면 된다.
///
/// # Warning
///
/// 최상위 `struct`로 두면 xtask가 스킬 구조체로 오인해 `Skill` enum에 넣는다. 모듈 안에 둘 것.
pub mod params {
    use crate::skill::Region;
    /// 계수는 전부 백분율이라 분모가 고정.
    pub const PERCENT_DEN: u16 = 100;
    pub const ACC_DAMAGE_CAP_PERCENT: u16 = 5000;
    #[derive(Debug, Clone, Copy)]
    pub struct ExParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub region: Region,
        /// 자신을 뺀 버프 대상 수.
        pub ally_count: u8,
        pub atk_buff_scale: u16,
        /// 83.8 반올림.
        pub effective_buff_scale: u16,
    }
    #[derive(Debug, Clone, Copy)]
    pub struct BasicParams {
        pub frames: u16,
        pub coef_percent: u16,
    }
    #[derive(Debug, Clone, Copy)]
    pub struct SubParams {
        pub duration: u16,
    }
}
/// 증폭 장치를 설치하여 원형범위 내에 있는 아군의 공격력 26.8 → 51% 증가,
/// 신비 특효 44.1 → 83.8% 가산 (25초간)
#[derive(Debug)]
pub struct KeiExSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    name: String,
    id: (u32, u8),
    params: params::ExParams,
}
/// 증폭 장치 작동 종료 시 적 1인에게 공격력 148 → 281% 대미지
/// 추가로 해당 증폭 장치 저장량의 40 → 100%만큼 대미지
/// (이 대미지는 치명 공격이 발생하지 않으며, 케이의 능력치에 영향받지 않습니다.)
#[derive(Debug)]
pub struct KeiBasicSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
    params: params::BasicParams,
}
/// 증폭 장치 작동 시작 시 증폭 장치 범위 내의 아군에게 치명 수치 13.1 → 22.3% 증가 (25초간)
/// 증폭 장치 작동 종료 시, 자신을 제외한 아군이 해당 증폭 장치 범위 내에서
/// 적에게 가한 대미지의 10%를 저장 (케이 기본 공격력의 5000%까지)
/// (저장량은 덮어씌워집니다)
#[derive(Debug)]
pub struct KeiSubSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
    params: params::SubParams,
}
impl KeiExSkill {
    pub fn new(
        name: &str,
        owner: &Student,
        skill_mask_offset: usize,
        params: params::ExParams,
    ) -> Self {
        Self {
            kei: NonNull::from(owner),
            skill_mask_offset,
            name: name.to_string(),
            params,
            id: (owner.id(), 0),
        }
    }
}
impl SkillOps for KeiExSkill {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn cost(&self) -> u8 {
        self.params.cost
    }
    fn duration(&self) -> u16 {
        self.params.duration
    }
    fn frames(&self) -> u16 {
        self.params.frames
    }
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    fn skill_type(&self) -> crate::skill::SkillType {
        SkillType::Ex
    }
    fn skill_effects(&self) -> Vec<crate::skill::SkillEffect> {
        let effective_buff = EffectKind::Buff {
            ty: StatKind::MysticEffectiveness,
            duration: self.params.duration,
            scale: self.params.effective_buff_scale,
            amount: 0,
        };
        let atk_buff = EffectKind::Buff {
            ty: StatKind::Atk,
            duration: self.params.duration,
            scale: self.params.atk_buff_scale,
            amount: 0,
        };
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: self.duration(),
            },
            targets: vec![
                SkillEffectTarget::Oneself {
                    kind: effective_buff,
                },
                SkillEffectTarget::Oneself { kind: atk_buff },
                SkillEffectTarget::Student {
                    kind: effective_buff,
                    count: self.params.ally_count,
                },
                SkillEffectTarget::Student {
                    kind: atk_buff,
                    count: self.params.ally_count,
                },
            ],
        }]
    }
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        let caster_coord = caster.coordinate;
        for target in targets.into_iter() {
            if is_inside(target.coordinate, self.params.region, caster_coord) {
                let already_applied =
                    (target.effects.0 & (0x01u64 << self.skill_mask_offset())) != 0;
                if !already_applied {
                    target.remained_effects.push(Reverse(RemainedEffects {
                        ticks: self.duration(),
                        offset: self.skill_mask_offset as u8,
                    }));
                    target.effects =
                        (target.effects.0 | (0x01u64 << self.skill_mask_offset)).into();
                }
            }
        }
        let already_applied = (caster.effects.0 & (0x01u64 << self.skill_mask_offset())) != 0;
        if !already_applied {
            caster.remained_effects.push(Reverse(RemainedEffects {
                ticks: self.duration(),
                offset: self.skill_mask_offset as u8,
            }));
            caster.effects = (caster.effects.0 | (0x01u64 << self.skill_mask_offset)).into();
        }
    }
}
impl KeiBasicSkill {
    pub fn new(
        name: &str,
        owner: &Student,
        skill_mask_offset: usize,
        params: params::BasicParams,
    ) -> Self {
        Self {
            kei: NonNull::from_ref(owner),
            skill_mask_offset,
            id: (owner.id(), 1),
            name: name.to_string(),
            params,
        }
    }
}
impl SkillOps for KeiBasicSkill {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn cost(&self) -> u8 {
        0
    }
    fn duration(&self) -> u16 {
        0
    }
    fn frames(&self) -> u16 {
        self.params.frames
    }
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    fn skill_type(&self) -> crate::skill::SkillType {
        SkillType::Basic
    }
    fn skill_effects(&self) -> Vec<crate::skill::SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::Damage {
                    coef_num: self.params.coef_percent,
                    coef_den: params::PERCENT_DEN,
                },
            }],
        }]
    }
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        assert_eq!(targets.len(), 1);
        let damage_key = caster.effects;
        for target in targets.into_iter() {
            if target.character.is_boss() {
                target.accumulated_damage.push(AccumulatedDamage {
                    ticks: 1,
                    damage: caster
                        .damage_map
                        .get(
                            &(damage_key.clone_with_tag(true, false, true)
                                | (0x01u64 << self.skill_mask_offset)),
                        )
                        .copied(),
                });
            }
        }
    }
}
impl KeiSubSkill {
    pub fn new(
        name: &str,
        owner: &Student,
        skill_mask_offset: usize,
        params: params::SubParams,
    ) -> Self {
        Self {
            kei: NonNull::from_ref(owner),
            skill_mask_offset,
            id: (owner.id(), 2),
            name: name.to_string(),
            params,
        }
    }
    pub fn effect_apply<'a>(skill: &Skill, mut state: State<'a>) -> State<'a> {
        let len = state.boss().accumulated_damage.len();
        let kei = skill.owner();
        let prior_idx = state
            .state_data_by_id(kei.id())
            .expect("cannot found kei")
            .extra_as::<KeiState>()
            .recording_start_len;
        let mut acc = 0;
        for i in prior_idx..len {
            if let Some(d) = state.boss().accumulated_damage[i].damage {
                acc += d.expected_value();
            }
        }
        let ex = state
            .state_data_by_id_mut(kei.id())
            .expect("cannot found kei")
            .extra_as_mut::<KeiState>();
        ex.acc_damage += acc;
        ex.recording_start_len = len;
        state
    }
}
impl SkillOps for KeiSubSkill {
    fn name(&self) -> &str {
        &self.name
    }
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    fn cost(&self) -> u8 {
        0
    }
    fn frames(&self) -> u16 {
        0
    }
    fn duration(&self) -> u16 {
        self.params.duration
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Sub
    }
    fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: self.duration(),
            },
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::new_other(Self::effect_apply),
            }],
        }]
    }
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        let cap = caster.character.stats().atk as u64 * params::ACC_DAMAGE_CAP_PERCENT as u64
            / params::PERCENT_DEN as u64;
        let acc_damage = {
            let extras = caster.extra_as::<KeiState>();
            extras.acc_damage.min(cap)
        };
        let damage = Damage::new(acc_damage, acc_damage, acc_damage, acc_damage, 0, 1, 0);
        for target in targets.into_iter() {
            target.accumulated_damage.push(AccumulatedDamage {
                ticks: 1,
                damage: Some(damage),
            });
        }
        caster.extra_as_mut::<KeiState>().acc_damage = 0;
    }
}
