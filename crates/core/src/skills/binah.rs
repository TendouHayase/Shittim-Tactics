use crate::{
    boss::Boss,
    character::Character,
    difficulty::Difficulty,
    skill::{
        DebuffType::Def, EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget,
        SkillType::Ex,
    },
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;
#[derive(Debug)]
pub struct BinahAtsilutsLight {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl BinahAtsilutsLight {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn cost(&self) -> u8 {
        0
    }
    pub fn duration(&self) -> u16 {
        0
    }
    pub fn frames(&self) -> u16 {
        todo!()
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.index
    }
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        let duration: u16;
        match unsafe { self.parent.read().stats.difficulty } {
            Difficulty::Torment => duration = 15 * 30,
            Difficulty::Lunatic => duration = 120 * 30,
            _ => duration = 0,
        }
        vec![
            SkillEffect {
                id: self.id,
                timing: EffectTiming::Instant,
                targets: vec![SkillEffectTarget::Land {
                    kind: EffectKind::new_damage(),
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
                    kind: EffectKind::new_damage(),
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
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        _caster: &'b StateData<'a>,
        _targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        todo!()
    }
    pub fn owner(&self) -> Character {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        crate::skill::SkillType::Ex
    }
}
impl BinahAtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        BinahAtsilutsLight {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 0),
            name: binah.stats.name.to_string(),
        }
    }
}
#[derive(Debug)]
pub struct BinahFiresofSeverity1 {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl BinahFiresofSeverity1 {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn cost(&self) -> u8 {
        0
    }
    pub fn duration(&self) -> u16 {
        0
    }
    pub fn frames(&self) -> u16 {
        todo!()
    }
    pub fn owner(&self) -> Character {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.index
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        crate::skill::SkillType::Ex
    }
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::new_damage(),
                count: 4,
            }],
        }]
    }
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let dmg_num;
        let dmg_den;
        match unsafe { self.parent.read().stats.difficulty } {
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
                    damage: target.damage_map.get(&target.effects).copied(),
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
                target.extra,
            ));
        }
        result
    }
}
impl BinahFiresofSeverity1 {
    const NAME: &str = "Fire of Severity 1";
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        BinahFiresofSeverity1 {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 1),
            name: binah.stats.name.to_string(),
        }
    }
}
#[derive(Debug)]
pub struct BinahFireofSeverity2 {
    parent: NonNull<Boss>,
    index: usize,
    name: String,
    id: (u32, u8),
}
impl BinahFireofSeverity2 {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn owner(&self) -> Character {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    pub fn cost(&self) -> u8 {
        0
    }
    pub fn duration(&self) -> u16 {
        0
    }
    pub fn frames(&self) -> u16 {
        todo!()
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.index
    }
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::new_damage(),
                count: 4,
            }],
        }]
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        Ex
    }
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
        let dmg_num;
        let mut dmg_den;
        match unsafe { self.parent.read().stats.difficulty } {
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
                    damage: target.damage_map.get(&target.effects).copied(),
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
                target.extra,
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
impl BinahFireofSeverity2 {
    const NAME: &str = "Fires of Severity 2";
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        Self {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            name: Self::NAME.to_string(),
            id: (binah.id(), 2),
        }
    }
}
#[derive(Debug)]
pub struct BinahPurifyingStorm {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl BinahPurifyingStorm {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn cost(&self) -> u8 {
        3
    }
    pub fn duration(&self) -> u16 {
        30
    }
    pub fn frames(&self) -> u16 {
        todo!()
    }
    pub fn skill_mask_offset(&self) -> usize {
        self.index
    }
    pub fn owner(&self) -> Character {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    pub fn skill_type(&self) -> crate::skill::SkillType {
        crate::skill::SkillType::Ex
    }
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::new_debuff(Def, 90, 50, 0),
                count: 4,
            }],
        }]
    }
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
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
                    damage: target.damage_map.get(&target.effects).copied(),
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
                target.extra,
            ));
        }
        result
    }
}
impl BinahPurifyingStorm {
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        Self {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 3),
            name: binah.stats.name.to_string(),
        }
    }
}
