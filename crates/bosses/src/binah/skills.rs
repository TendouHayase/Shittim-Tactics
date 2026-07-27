use core::{
    boss::Boss,
    character::Character,
    difficulty::Difficulty,
    skill::{
        DebuffType::Def, EffectKind, EffectTiming, Region, Skill, SkillEffect, SkillEffectTarget,
        SkillType,
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
impl AtsilutsLight {
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
    pub fn apply<'a: 'b, 'b>(
        &self,
        caster: &'b mut StateData<'a>,
        targets: &'b mut [StateData<'a>],
    ) -> &'b mut [StateData<'a>] {
        todo!()
    }
    pub fn owner(&self) -> Character {
        unsafe { Character::Boss(self.parent.as_ref()) }
    }
    pub fn skill_type(&self) -> SkillType {
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
impl FiresofSeverity1 {
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
    pub fn skill_type(&self) -> SkillType {
        SkillType::Ex
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
    pub fn apply<'a: 'b, 'b>(
        &self,
        caster: &'b mut StateData<'a>,
        targets: &'b mut [StateData<'a>],
    ) -> &'b mut [StateData<'a>] {
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
        targets
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
impl FireofSeverity2 {
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
    pub fn skill_type(&self) -> SkillType {
        SkillType::Ex
    }
    pub fn apply<'a: 'b, 'b>(
        &self,
        caster: &'b mut StateData<'a>,
        targets: &'b mut [StateData<'a>],
    ) -> &'b mut [StateData<'a>] {
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
        targets
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
impl PurifyingStorm {
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
    pub fn skill_type(&self) -> SkillType {
        SkillType::Ex
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
    pub fn apply<'a: 'b, 'b>(
        &self,
        caster: &'b mut StateData<'a>,
        targets: &'b mut [StateData<'a>],
    ) -> &'b mut [StateData<'a>] {
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
        targets
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
