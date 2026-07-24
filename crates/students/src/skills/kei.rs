use core::{
    character::Character,
    damage::{Damage, key::SkillsBitMask},
    skill::{
        BuffType::{self, Def},
        EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget, SkillType,
    },
    state::{AccumulatedDamage, RemainedEffects, StateData, Stateful},
    student::Student,
    types::AttackType,
    utils::{TPS, is_inside},
};
use std::{
    any::Any,
    cmp::Reverse,
    collections::HashMap,
    rc,
    sync::{Arc, Mutex, Weak},
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

#[derive(Debug, Clone, Default)]
pub struct SubSkillState {
    /// 서브 스킬 효과로 누적된 데미지
    pub acc_damage: u64,

    /// 데미지 기록 직전까지 누적된 데미지 기록
    pub recording_start_len: usize,
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
                let already_applied = (target.effects.0 & self.skill_mask_offset() as u64) != 0;
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
                            effects: (target.effects.0 | (0x80u64 >> self.skill_mask_offset))
                                .into(),
                            remained_effects,
                            accumulated_damage: target.accumulated_damage.clone(),
                            damage_map: target.damage_map,
                            extra: target.extra,
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
            ticks: 1,
            damage: caster
                .damage_map
                .get(
                    &(damage_key.clone_with_tag(true, false, true) | self.skill_mask_offset as u64)
                        .into(),
                )
                .copied(),
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

impl Skill<SubSkillState> for SubSkill {
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
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::Other(SubSkill::effect_apply),
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let atk = caster.character.stats().atk * 50;

        let acc_damage = {
            let extras = caster.extra_as::<SubSkillState>();
            extras.acc_damage.min(atk.into())
        };

        let mut result = vec![];

        let damage = Damage::new(acc_damage, acc_damage, acc_damage, acc_damage, 0, 1, 0);

        for &target in targets {
            if target.character.id() != caster.character.id() {
                let mut target_clone = target.clone();
                target_clone.accumulated_damage.push(AccumulatedDamage {
                    ticks: 1,
                    damage: Some(damage),
                });
                result.push(target_clone);
            }
        }
        let mut caster_clone = caster.clone();
        caster_clone.extra_as_mut::<SubSkillState>().acc_damage = 0;
        result.push(caster_clone);

        result
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

    pub fn effect_apply<'a, T: Skill, S: Stateful<'a>>(skill: &T, state: S) -> S {
        let kei;
        match skill.owner().upgrade() {
            Some(k) => kei = k,
            None => panic!("cannot found Kei"),
        }

        let mut state_clone = state.clone();

        let kei_state = state_clone
            .state_data_by_id_mut(kei.id())
            .expect("cannot found kei");

        let ex = kei_state.extra_as_mut::<SubSkillState>();
        let prior_idx = ex.recording_start_len;
        for i in prior_idx..state.boss().accumulated_damage.len() {
            match state.boss().accumulated_damage[i].damage {
                Some(d) => ex.acc_damage += d.expected_value(),
                None => (),
            }
        }

        state_clone
    }
}
