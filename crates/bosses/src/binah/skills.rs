use crate::create_boss_skill;
use core::{
    boss::Boss,
    character::Character,
    difficulty::Difficulty,
    skill::{
        DebuffType::Def, EffectKind, EffectTiming, Region, SkillEffect, SkillEffectTarget, SkillOps,
        SkillType,
    },
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;

create_boss_skill!(AtsilutsLight, 0, 0, todo!(), SkillType::Ex, 0, {
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
});

impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";
}

create_boss_skill!(FiresofSeverity1, 0, 0, todo!(), SkillType::Ex, 1, {
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
});

impl FiresofSeverity1 {
    const NAME: &str = "Fire of Severity 1";
}

create_boss_skill!(FireofSeverity2, 0, 0, todo!(), SkillType::Ex, 2, {
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
                    damage: Some(target.damage_map[&target.effects] * dmg_den / dmg_num),
                });
            }

            if i == 0 {
                dmg_den *= 2;
            } else if i == 2 {
                dmg_den *= 2;
            }
        }
    }
});

impl FireofSeverity2 {
    const NAME: &str = "Fires of Severity 2";
}

create_boss_skill!(PurifyingStorm, 3, 30, todo!(), SkillType::Ex, 3, {
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
});
