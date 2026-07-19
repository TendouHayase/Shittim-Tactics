use core::{
    TPS,
    character::Character,
    damage::key::DamageKey,
    skill::{
        BuffType::{self},
        EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget, SkillType,
    },
    state::{AccumulatedDamage, RemainedEffects, StateData},
    student::Student,
    types::AttackType,
    utils::is_inside,
};
use std::{
    cmp::Reverse,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct ExSkill {
    kei: Weak<Student>,
    skill_mask_offset: usize,
    name: String,
    id: (u32, u8),
    effective_buff_scale: u16,
    atk_buff_scale: u16,
}

#[derive(Debug)]
pub struct BasicSkill {
    kei: Weak<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
}

#[derive(Debug)]
pub struct SubSkill {
    kei: Weak<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
    accumulated_damage: u64,
}

impl Skill for ExSkill {
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

    fn owner(&self) -> std::sync::Weak<dyn core::character::Character> {
        self.kei.clone()
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
            duration: 25 * TPS,
            scale: self.effective_buff_scale, // 83.8 반올림
            amount: 0,
        };

        let atk_buff = EffectKind::Buff {
            ty: BuffType::Atk,
            duration: 25 * TPS,
            scale: self.atk_buff_scale,
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
        caster: &'b core::state::StateData<'a>,
        targets: &'b [&'c core::state::StateData<'a>],
    ) -> Vec<core::state::StateData<'a>> {
        let caster_coord = caster.coordinate;

        let mut result: Vec<StateData<'_>> = vec![];

        for &target in targets {
            if is_inside(target.coordinate, Self::REGION, caster_coord) {
                let already_applied =
                    (target.effects.mask.0 & self.skill_mask_offset() as u64) != 0;
                if already_applied {
                    result.push(target.clone());
                } else {
                    if target.character.id() == caster.character.id() {
                    } else {
                        let mut remained_effects = target.remained_effects.clone();
                        remained_effects.push(Reverse(RemainedEffects {
                            ticks: self.duration(),
                            bit: self.skill_mask_offset as u8,
                        }));
                        result.push(StateData {
                            character: target.character,
                            coordinate: target.coordinate,
                            accumulated_damage_cache: target.accumulated_damage_cache.clone(),
                            cooldowns: target.cooldowns.clone(),
                            effects: DamageKey::from_mask(
                                (target.effects.mask.0 | (0x80u64 >> self.skill_mask_offset))
                                    .into(),
                                &target.effects,
                            ),
                            remained_effects,
                            accumulated_damage: target.accumulated_damage.clone(),
                        });
                    }
                }
            }
        }

        result
    }
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
            kei: Arc::downgrade(&unsafe { Arc::from_raw(owner as *const Student) }),
            skill_mask_offset,
            name: name.to_string(),
            atk_buff_scale,
            effective_buff_scale,
            id: (owner.id(), 0),
        }
    }
}

impl Skill for BasicSkill {
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

    fn owner(&self) -> std::sync::Weak<dyn core::character::Character> {
        self.kei.clone()
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
                kind: EffectKind::Damage,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b core::state::StateData<'a>,
        targets: &'b [&'c core::state::StateData<'a>],
    ) -> Vec<core::state::StateData<'a>> {
        assert_eq!(targets.len(), 1); // 대상이 1명이 아니면 오류

        let damage_key = &caster.effects;

        let mut result: Vec<StateData<'_>> = targets.iter().map(|x| *x).cloned().collect();

        result[0].accumulated_damage.push(AccumulatedDamage {
            damage: damage_key.clone_with_tag(true, false, true) | self.skill_mask_offset as u64,
            ticks: 1,
        });

        result
    }
}

impl BasicSkill {
    pub fn new(name: &str, owner: &Student, skill_mask_offset: usize) -> Self {
        Self {
            kei: Arc::downgrade(&unsafe { Arc::from_raw(owner as *const Student) }),
            skill_mask_offset,
            id: (owner.id(), 1),
            name: name.to_string(),
        }
    }
}

impl Skill for SubSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn owner(&self) -> Weak<dyn Character> {
        self.kei.clone()
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

    fn skill_effects<'a>(&'a self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: self.duration(),
            },
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::Other,
                count: 5,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
    }
}

impl SubSkill {
    pub fn new(name: &str, owner: &Student, skill_mask_offset: usize) -> Self {
        Self {
            kei: Arc::downgrade(&unsafe { Arc::from_raw(owner as *const Student) }),
            skill_mask_offset,
            id: (owner.id(), 2),
            name: name.to_string(),
            accumulated_damage: 0,
        }
    }
}
