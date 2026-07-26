use crate::states::kei::SubSkillState;
use crate::{
    character::Character,
    damage::Damage,
    skill::{
        BuffType::{self},
        EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget, SkillType,
    },
    state::{AccumulatedDamage, RemainedEffects, State, StateData, Stateful},
    student::Student,
    types::AttackType,
    utils::{is_inside, TPS},
};
use std::{cmp::Reverse, ptr::NonNull};
#[derive(Debug)]
pub struct KeiExSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    name: String,
    id: (u32, u8),
    effective_buff_scale: u16,
    atk_buff_scale: u16,
}
#[derive(Debug)]
pub struct KeiBasicSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
}
#[derive(Debug)]
pub struct KeiSubSkill {
    kei: NonNull<Student>,
    skill_mask_offset: usize,
    id: (u32, u8),
    name: String,
}
impl KeiExSkill {
    const REGION: Region = Region::Arc {
        radius: 1050,
        start_angle_degree: 0,
        end_angle_degree: 360,
    };
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
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn cost(&self) -> u8 {
        2
    }
    pub fn duration(&self) -> u16 {
        25 * TPS
    }
    pub fn frames(&self) -> u16 {
        123
    }
    pub fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        SkillType::Ex
    }
    pub fn skill_effects(&self) -> Vec<crate::skill::SkillEffect> {
        let effective_buff = EffectKind::new_buff(
            BuffType::Effectiveness(AttackType::Mystic),
            25 * TPS,
            self.effective_buff_scale,
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
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b crate::state::StateData<'a>,
        targets: &'b [&'c crate::state::StateData<'a>],
    ) -> Vec<crate::state::StateData<'a>> {
        let caster_coord = caster.coordinate;
        let mut result: Vec<StateData<'_>> = vec![];
        for &target in targets {
            if is_inside(target.coordinate, Self::REGION, caster_coord) {
                let already_applied =
                    (target.effects.0 & (0x80u64 >> self.skill_mask_offset())) != 0;
                if already_applied {
                    result.push(target.clone());
                } else {
                    if target.character.id() != caster.character.id() {
                        let mut remained_effects = target.remained_effects.clone();
                        remained_effects.push(Reverse(RemainedEffects {
                            ticks: self.duration(),
                            offset: self.skill_mask_offset as u8,
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
impl KeiBasicSkill {
    pub fn new(name: &str, owner: &Student, skill_mask_offset: usize) -> Self {
        Self {
            kei: NonNull::from_ref(owner),
            skill_mask_offset,
            id: (owner.id(), 1),
            name: name.to_string(),
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn cost(&self) -> u8 {
        0
    }
    pub fn duration(&self) -> u16 {
        0
    }
    pub fn frames(&self) -> u16 {
        141
    }
    pub fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        SkillType::Basic
    }
    pub fn skill_effects(&self) -> Vec<crate::skill::SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::new_damage(),
            }],
        }]
    }
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b crate::state::StateData<'a>,
        targets: &'b [&'c crate::state::StateData<'a>],
    ) -> Vec<crate::state::StateData<'a>> {
        assert_eq!(targets.len(), 1);
        let damage_key = &caster.effects;
        let mut result: Vec<StateData<'_>> = targets.iter().copied().cloned().collect();
        result[0].accumulated_damage.push(AccumulatedDamage {
            ticks: 1,
            damage: caster
                .damage_map
                .get(
                    &(damage_key.clone_with_tag(true, false, true)
                        | (0x80u64 >> self.skill_mask_offset)),
                )
                .copied(),
        });
        result
    }
}
impl KeiSubSkill {
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
            .extra_as::<SubSkillState>()
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
            .extra_as_mut::<SubSkillState>();
        ex.acc_damage += acc;
        ex.recording_start_len = len;
        state
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn owner(&self) -> Character<'_> {
        unsafe { Character::Student(self.kei.as_ref()) }
    }
    pub fn cost(&self) -> u8 {
        0
    }
    pub fn frames(&self) -> u16 {
        0
    }
    pub fn duration(&self) -> u16 {
        25 * TPS
    }
    pub fn skill_type(&self) -> SkillType {
        SkillType::Sub
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.skill_mask_offset
    }
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
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
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
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
