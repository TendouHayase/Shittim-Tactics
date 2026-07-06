use core::{
    damage::Damage,
    skill::{Debuff::Def, Effect, EffectKind, EffectTarget, EffectTiming, Skill},
};

use crate::difficulty::{self, Difficulty};

#[derive(Debug)]
pub struct AtsilutsLight {
    difficulty: Difficulty,
}

impl Skill for AtsilutsLight {
    fn cost(&self) -> u8 {
        0
    }

    fn frames(&self) -> u32 {
        1
    }

    fn effects(&self) -> Vec<Effect> {
        let duration: u32;

        match self.difficulty {
            Difficulty::Torment => duration = 15 * 60,
            Difficulty::Lunatic => duration = 120 * 60,
            _ => duration = 0,
        }

        vec![
            Effect {
                name: Self::SKILL_1,
                kind: EffectKind::Damage,
                timing: EffectTiming::Instant,
                targets: vec![],
            },
            Effect {
                name: Self::SKILL_2,
                kind: EffectKind::DamageRegion {
                    length: 2200,
                    width: 300,
                },
                timing: EffectTiming::Persistent {
                    interval_frames: 180,
                    duration_frames: duration,
                },
                targets: vec![],
            },
        ]
    }

    fn apply(&self, ctx: &mut core::skill::SkillContext) {
        let atk: u64 = ctx.caster.atk().into();
        let dmg_scale: f64;

        match self.difficulty {
            Difficulty::Torment => dmg_scale = 1.6,
            Difficulty::Lunatic => dmg_scale = 2.,
            _ => dmg_scale = 1.2,
        }

        for target in ctx.targets.iter_mut() {}
    }
}

impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";

    pub fn new(difficulty: Difficulty) -> Self {
        AtsilutsLight { difficulty }
    }
}

#[derive(Debug)]
pub struct FiresofSeverity {
    difficulty: Difficulty,
}

impl Skill for FiresofSeverity {
    fn cost(&self) -> u8 {
        0
    }

    fn frames(&self) -> u32 {
        1
    }

    fn effects(&self) -> Vec<Effect> {
        vec![
            Effect {
                name: Self::SKILL_1,
                kind: EffectKind::Damage,
                timing: EffectTiming::Instant,
                targets: vec![EffectTarget::Student; 4],
            },
            Effect {
                name: Self::SKILL_2,
                kind: EffectKind::Damage,
                timing: EffectTiming::Instant,
                targets: vec![EffectTarget::Student; 4],
            },
        ]
    }

    fn apply(&self, ctx: &mut core::skill::SkillContext) {
        let dmg_num;
        let mut dmg_den;

        if ctx.name == FiresofSeverity::SKILL_1 {
            match self.difficulty {
                Difficulty::Insane | Difficulty::Torment | Difficulty::Lunatic => {
                    dmg_num = 3;
                    dmg_den = 2;
                }
                _ => {
                    dmg_num = 3;
                    dmg_den = 4;
                }
            }
            for target in ctx.targets.iter_mut() {
                let d = Damage::from_skill_context(&ctx.caster, target, dmg_num, dmg_den);
                target.accumulated_damage.push(d);
                target.accumulated_damage_cached.append(&d);
            }
        } else if ctx.name == FiresofSeverity::SKILL_2 {
            match self.difficulty {
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
                ctx.targets.len(),
                4,
                "Fire of Severity 2 Skill is not a target of 4 people"
            );

            for (i, target) in ctx.targets.iter_mut().enumerate() {
                let d = Damage::from_skill_context(&ctx.caster, target, dmg_num, dmg_den);
                target.accumulated_damage.push(d);
                target.accumulated_damage_cached.append(&d);
                if i == 0 {
                    dmg_den *= 2;
                } else if i == 2 {
                    dmg_den *= 2;
                }
            }
        }
    }
}

impl FiresofSeverity {
    const SKILL_1: &str = "Fires of Severity 1";
    const SKILL_2: &str = "Fires of Severity 2";

    pub fn new(difficulty: Difficulty) -> Self {
        FiresofSeverity { difficulty }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PurifyingStorm {
    pub difficulty: Difficulty,
}

impl Skill for PurifyingStorm {
    fn cost(&self) -> u8 {
        3
    }

    fn frames(&self) -> u32 {
        todo!()
    }

    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            name: "PurifyingStorm",
            kind: EffectKind::Debuff {
                ty: Def,
                duration: 3000,
                scale: 50,
                amount: 0,
            },
            timing: EffectTiming::Instant,
            targets: vec![EffectTarget::Student; 4],
        }]
    }

    fn apply(&self, ctx: &mut core::skill::SkillContext) {
        for target in ctx.targets.iter_mut() {
            let dmg = Damage::from_skill_context(&ctx.caster, target, 300, 1);
            target.accumulated_damage.push(dmg);
            target.accumulated_damage_cached.append(&dmg);
        }
    }
}

impl PurifyingStorm {
    pub fn new(difficulty: Difficulty) -> Self {
        PurifyingStorm { difficulty }
    }
}
