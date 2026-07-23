use core::{
    character::Character,
    difficulty::Difficulty,
    skill::{
        DebuffType::Def, EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget,
        SkillType::Ex,
    },
    state::{AccumulatedDamage, StateData},
};
use std::sync::{Arc, Weak};

use crate::binah::Binah;

// TODO! 스킬 이름 런타임에 선택한 언어로 반환 구현 필요

#[derive(Debug)]
pub struct AtsilutsLight {
    parent: Weak<Binah>,
    index: usize,
    id: (u32, u8),
}

impl Skill for AtsilutsLight {
    fn name(&self) -> &str {
        "Atsilut's Light"
    }
    fn cost(&self) -> u8 {
        0
    }

    fn duration(&self) -> u16 {
        0
    }

    fn frames(&self) -> u16 {
        todo!()
    }

    fn skill_mask_offset(&self) -> usize {
        self.index
    }

    fn skill_effects(&self) -> Vec<SkillEffect> {
        let duration: u16;

        match self.parent.upgrade().unwrap().difficulty {
            Difficulty::Torment => duration = 15 * 30,
            Difficulty::Lunatic => duration = 120 * 30,
            _ => duration = 0,
        }

        vec![
            SkillEffect {
                id: self.id,
                timing: EffectTiming::Instant,
                targets: vec![SkillEffectTarget::Land {
                    kind: EffectKind::Damage,
                    region: Region::Polygon {
                        vertex: [
                            (-150, 2200).into(),
                            (150, 2200).into(),
                            (150, 0).into(),
                            (-150, 0).into(),
                        ],
                        count: 4,
                    },
                }],
            },
            SkillEffect {
                id: self.id,
                timing: EffectTiming::Persistent {
                    interval_frames: 90,
                    duration_frames: duration,
                },
                targets: vec![SkillEffectTarget::Land {
                    kind: EffectKind::Damage,
                    region: Region::Polygon {
                        vertex: [
                            (-150f32, 2200f32).into(),
                            (150, 2200).into(),
                            (150, 0).into(),
                            (-150, 0).into(),
                        ],
                        count: 4,
                    },
                }],
            },
        ]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        _caster: &'b StateData<'a>,
        _targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        todo!()
    }

    fn owner(&self) -> Weak<dyn Character> {
        self.parent.clone()
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }
}

impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";

    pub fn new(binah: &Binah, skill_mask_index: usize) -> Self {
        AtsilutsLight {
            parent: Arc::downgrade(&unsafe { Arc::from_raw(binah as *const Binah) }),
            index: skill_mask_index,
            id: (binah.id(), 0),
        }
    }
}

#[derive(Debug)]
pub struct FiresofSeverity1 {
    parent: Weak<Binah>,
    index: usize,
    id: (u32, u8),
}

impl Skill for FiresofSeverity1 {
    fn name(&self) -> &str {
        "Fire of Severity"
    }
    fn cost(&self) -> u8 {
        0
    }

    fn duration(&self) -> u16 {
        0
    }

    fn frames(&self) -> u16 {
        todo!()
    }

    fn owner(&self) -> Weak<dyn Character> {
        self.parent.clone()
    }

    fn skill_mask_offset(&self) -> usize {
        self.index
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::Damage,
                count: 4,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let dmg_num;
        let dmg_den;

        match self.parent.upgrade().unwrap().difficulty {
            Difficulty::Insane | Difficulty::Torment | Difficulty::Lunatic => {
                dmg_num = 3;
                dmg_den = 2;
            }
            _ => {
                dmg_num = 3;
                dmg_den = 4;
            }
        }

        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for &target in targets.iter() {
            let d = caster.damage_with_effects();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * dmg_num / dmg_den));
                ac_dmg.push(AccumulatedDamage {
                    ticks: self.duration(),
                    damage: target.damage_map.get(&target.effects.into()).copied(),
                });
            }

            result.push(StateData::from_parts(
                target.character,
                target.coordinate,
                &target.cooldowns,
                &target.effects,
                &target.remained_effects,
                &ac_dmg,
                ac_dmg_cache,
                target.damage_map,
            ));
        }

        result
    }
}

impl FiresofSeverity1 {
    const NAME: &str = "Fire of Severity 1";

    pub fn new(binah: &Binah, skill_mask_index: usize) -> Self {
        FiresofSeverity1 {
            parent: Arc::downgrade(&unsafe { Arc::from_raw(binah as *const Binah) }),
            index: skill_mask_index,
            id: (binah.id(), 1),
        }
    }
}

#[derive(Debug)]
pub struct FireofSeverity2 {
    parent: Weak<Binah>,
    index: usize,
    name: String,
    id: (u32, u8),
}

impl Skill for FireofSeverity2 {
    fn name(&self) -> &str {
        &self.name
    }

    fn owner(&self) -> Weak<dyn Character> {
        self.parent.clone()
    }

    fn cost(&self) -> u8 {
        0
    }

    fn duration(&self) -> u16 {
        0
    }

    fn frames(&self) -> u16 {
        todo!()
    }

    fn skill_mask_offset(&self) -> usize {
        self.index
    }

    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::Damage,
                count: 4,
            }],
        }]
    }

    fn skill_type(&self) -> core::skill::SkillType {
        Ex
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let dmg_num;
        let mut dmg_den;

        match self.parent.upgrade().unwrap().difficulty {
            Difficulty::Insane | Difficulty::Torment | Difficulty::Lunatic => {
                dmg_num = 15;
                dmg_den = 2;
            }
            _ => {
                dmg_num = 15;
                dmg_den = 4;
            }
        }

        assert_eq!(
            targets.len(),
            4,
            "Fire of Severity 2 Skill is not a target of 4 people"
        );

        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for (i, &target) in targets.iter().enumerate() {
            let d = caster.damage_with_effects();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * dmg_num / dmg_den));
                ac_dmg.push(AccumulatedDamage {
                    ticks: self.duration(),
                    damage: target.damage_map.get(&target.effects.into()).copied(),
                });
            }

            result.push(StateData::from_parts(
                target.character,
                target.coordinate,
                &target.cooldowns,
                &target.effects,
                &target.remained_effects,
                &ac_dmg,
                ac_dmg_cache,
                target.damage_map,
            ));

            if i == 0 {
                dmg_den *= 2;
            } else if i == 2 {
                dmg_den *= 2;
            }
        }

        result
    }
}

impl FireofSeverity2 {
    const NAME: &str = "Fires of Severity 2";

    pub fn new(binah: &Binah, skill_mask_index: usize) -> Self {
        Self {
            parent: Arc::downgrade(&unsafe { Arc::from_raw(binah as *const Binah) }),
            index: skill_mask_index,
            name: Self::NAME.to_string(),
            id: (binah.id(), 2),
        }
    }
}

#[derive(Debug)]
pub struct PurifyingStorm {
    parent: Weak<Binah>,
    index: usize,
    id: (u32, u8),
}

impl Skill for PurifyingStorm {
    fn name(&self) -> &str {
        "Purifying Storm"
    }
    fn cost(&self) -> u8 {
        3
    }

    fn duration(&self) -> u16 {
        30
    }

    fn frames(&self) -> u16 {
        todo!()
    }

    fn skill_mask_offset(&self) -> usize {
        self.index
    }

    fn owner(&self) -> Weak<dyn Character> {
        self.parent.clone()
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::Debuff {
                    ty: Def,
                    duration: 90,
                    scale: 50,
                    amount: 0,
                },
                count: 4,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for &target in targets.iter() {
            let d = caster.damage_with_effects();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * 3));
                ac_dmg.push(AccumulatedDamage {
                    damage: target.damage_map.get(&target.effects.into()).copied(),
                    ticks: self.duration(),
                });
            }

            result.push(StateData::from_parts(
                target.character,
                target.coordinate,
                &target.cooldowns,
                &target.effects,
                &target.remained_effects,
                &ac_dmg,
                ac_dmg_cache,
                target.damage_map,
            ));
        }

        result
    }
}

impl PurifyingStorm {
    pub fn new(binah: &Binah, skill_mask_index: usize) -> Self {
        Self {
            parent: Arc::downgrade(&unsafe { Arc::from_raw(binah as *const Binah) }),
            index: skill_mask_index,
            id: (binah.id(), 3),
        }
    }
}
