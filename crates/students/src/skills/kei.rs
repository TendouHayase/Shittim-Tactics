use crate::states::KeiState;
use core::{
    character::Character,
    damage::Damage,
    skill::{
        BuffType::{self},
        EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget, SkillOps,
        SkillType,
    },
    state::{AccumulatedDamage, RemainedEffects, State, StateData, Stateful},
    student::Student,
    types::AttackType,
    utils::{TPS, is_inside},
};
use std::{cmp::Reverse, ptr::NonNull};

#[derive(Debug)]
pub struct ExSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    name: String,
    id: (u32, u8),
    effective_buff_scale: u16,
    atk_buff_scale: u16,
}

#[derive(Debug)]
pub struct BasicSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
}

#[derive(Debug)]
pub struct SubSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
}

impl ExSkill {
    const REGION: Region = Region::Arc {
        radius: 1050,
        start_angle_degree: 0,
        end_angle_degree: 360,
    };

    // 반드시 수명을 학생 객체와 맞출것
    pub fn new(
        name: &str,
        owner: &Student,
        skill_mask_offset: usize,
        atk_buff_scale: u16,
        effective_buff_scale: u16,
    ) -> Self {
        Self {
            kei: NonNull::from(owner),
            skill_mask_offset,
            name: name.to_string(),
            atk_buff_scale,
            effective_buff_scale,
            id: (owner.id(), 0),
        }
    }
}

impl SkillOps for ExSkill {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn cost(&self) -> u8 {
        2
    }

    fn duration(&self) -> u16 {
        25 * TPS
    }

    fn frames(&self) -> u16 {
        123
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
        let effective_buff = EffectKind::new_buff(
            BuffType::Effectiveness(AttackType::Mystic),
            25 * TPS,
            self.effective_buff_scale, // 83.8 반올림
            0,
        );

        let atk_buff = EffectKind::new_buff(BuffType::Atk, 25 * TPS, self.atk_buff_scale, 0);

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
                    count: 6,
                },
                SkillEffectTarget::Student {
                    kind: atk_buff,
                    count: 6,
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
            if is_inside(target.coordinate, Self::REGION, caster_coord) {
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
    pub fn new(name: &str, owner: &Student, skill_mask_offset: usize) -> Self {
        Self {
            kei: NonNull::from_ref(owner),
            skill_mask_offset,
            id: (owner.id(), 1),
            name: name.to_string(),
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
        141
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
                kind: EffectKind::new_damage(),
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
    pub fn new(name: &str, owner: &Student, skill_mask_offset: usize) -> Self {
        Self {
            kei: NonNull::from_ref(owner),
            skill_mask_offset,
            id: (owner.id(), 2),
            name: name.to_string(),
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
        25 * TPS
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
        let atk = caster.character.stats().atk * 50;

        let acc_damage = {
            let extras = caster.extra_as::<KeiState>();
            extras.acc_damage.min(atk.into())
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
