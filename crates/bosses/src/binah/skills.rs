use core::{
    boss::Boss,
    character::Character,
    difficulty::Difficulty,
    skill::{
        DebuffType::Def, EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget,
        SkillOps, SkillType,
    },
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;
#[derive(Debug)]
pub struct AtsilutsLight {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl SkillOps for AtsilutsLight {
    fn name(&self) -> &str {
        &self.name
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
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        todo!()
    }
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Ex
    }
}
impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        Self {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 0),
            name: binah.stats.name.to_string(),
        }
    }
}
#[derive(Debug)]
pub struct FiresofSeverity1 {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl SkillOps for FiresofSeverity1 {
    fn name(&self) -> &str {
        &self.name
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
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    fn skill_mask_offset(&self) -> usize {
        self.index
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Ex
    }
    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::new_damage(),
                count: 4,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
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
        for target in targets.iter_mut() {
            let d = caster.damage_with_effects();
            if let Some(damage) = d {
                target
                    .accumulated_damage_cache
                    .append(&(damage * dmg_num / dmg_den));
                target.accumulated_damage.push(AccumulatedDamage {
                    ticks: self.duration(),
                    damage: target.damage_map.get(&target.effects).copied(),
                });
            }
        }
    }
}
impl FiresofSeverity1 {
    const NAME: &str = "Fire of Severity 1";
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        Self {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 1),
            name: binah.stats.name.to_string(),
        }
    }
}
#[derive(Debug)]
pub struct FireofSeverity2 {
    parent: NonNull<Boss>,
    index: usize,
    name: String,
    id: (u32, u8),
}
impl SkillOps for FireofSeverity2 {
    fn name(&self) -> &str {
        &self.name
    }
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Boss(self.parent.as_ref()) }
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
                kind: EffectKind::new_damage(),
                count: 4,
            }],
        }]
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Ex
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
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
        for (i, target) in targets.iter_mut().enumerate() {
            let d = caster.damage_with_effects();
            if let Some(damage) = d {
                target
                    .accumulated_damage_cache
                    .append(&(damage * dmg_num / dmg_den));
                target.accumulated_damage.push(AccumulatedDamage {
                    ticks: self.duration(),
                    damage: target.damage_map.get(&target.effects).copied(),
                });
            }

            if i == 0 {
                dmg_den *= 2;
            } else if i == 2 {
                dmg_den *= 2;
            }
        }
    }
}
impl FireofSeverity2 {
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
pub struct PurifyingStorm {
    parent: NonNull<Boss>,
    index: usize,
    id: (u32, u8),
    name: String,
}
impl SkillOps for PurifyingStorm {
    fn name(&self) -> &str {
        &self.name
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
    fn owner(&self) -> Character<'_> {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Ex
    }
    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student {
                kind: EffectKind::new_debuff(Def, 90, 50, 0),
                count: 4,
            }],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        for target in targets.iter_mut() {
            let d = caster.damage_with_effects();
            if let Some(damage) = d {
                target.accumulated_damage_cache.append(&(damage * 3));
                target.accumulated_damage.push(AccumulatedDamage {
                    damage: target.damage_map.get(&target.effects).copied(),
                    ticks: self.duration(),
                });
            }
        }
    }
}
impl PurifyingStorm {
    pub fn new(binah: &Boss, skill_mask_index: usize) -> Self {
        Self {
            parent: NonNull::from_ref(binah),
            index: skill_mask_index,
            id: (binah.id(), 3),
            name: binah.stats.name.to_string(),
        }
    }
}
