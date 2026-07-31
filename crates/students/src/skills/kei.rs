use crate::states::KeiState;
use core::{
    character::Character,
    damage::Damage,
    skill::{
        BuffType::{self},
        EffectKind, EffectTiming, Skill, SkillEffect, SkillEffectTarget, SkillOps, SkillType,
    },
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
    use core::skill::Region;

    /// 계수는 전부 백분율이라 분모가 고정.
    pub const PERCENT_DEN: u16 = 100;

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
        /// 누적 데미지 상한을 공격력의 몇 %로 둘지.
        pub acc_damage_cap_percent: u16,
    }
}

#[derive(Debug)]
pub struct ExSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    name: String,
    id: (u32, u8),
    params: params::ExParams,
}

#[derive(Debug)]
pub struct BasicSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
    params: params::BasicParams,
}

#[derive(Debug)]
pub struct SubSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
    params: params::SubParams,
}

impl ExSkill {
    // 반드시 수명을 학생 객체와 맞출것
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

impl SkillOps for ExSkill {
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

    fn skill_type(&self) -> core::skill::SkillType {
        SkillType::Ex
    }

    fn skill_effects(&self) -> Vec<core::skill::SkillEffect> {
        let effective_buff = EffectKind::Buff {
            ty: BuffType::Effectiveness(AttackType::Mystic),
            duration: self.params.duration,
            scale: self.params.effective_buff_scale,
            amount: 0,
        };

        let atk_buff = EffectKind::Buff {
            ty: BuffType::Atk,
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

impl BasicSkill {
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

impl SkillOps for BasicSkill {
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

    fn skill_type(&self) -> core::skill::SkillType {
        SkillType::Basic
    }

    fn skill_effects(&self) -> Vec<core::skill::SkillEffect> {
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
        assert_eq!(targets.len(), 1); // 대상이 1명이 아니면 오류

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

impl SubSkill {
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

impl SkillOps for SubSkill {
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
        let cap = caster.character.stats().atk as u64 * self.params.acc_damage_cap_percent as u64
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
