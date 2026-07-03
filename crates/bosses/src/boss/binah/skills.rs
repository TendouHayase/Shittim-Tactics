use core::skill::{Effect, EffectKind, EffectTarget, EffectTiming, Skill};

use crate::difficulty::Difficulty;

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

    fn apply(&self, ctx: &mut core::context::SkillContext) {
        let atk: u64 = ctx.caster.atk().into();
        let dmg: u64;

        match self.difficulty {
            Difficulty::Torment => dmg = (atk as f32 * 1.6) as u64,
            Difficulty::Lunatic => dmg = atk * 2,
            _ => dmg = (atk as f32 * 1.2) as u64,
        }

        for target in ctx.targets.iter_mut() {
            target.decrease_hp(dmg);
        }
    }
}

impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";
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

    fn apply(&self, ctx: &mut core::context::SkillContext) {
        let atk: u64 = ctx.caster.atk() as u64;
        let dmg: u64;

        if ctx.name == Self::SKILL_1 {
            match self.difficulty {
                Difficulty::Insane => dmg = atk * 3 / 2,
                Difficulty::Torment => dmg = atk * 3 / 2,
                Difficulty::Lunatic => dmg = atk * 3 / 2,
                _ => dmg = atk * 3 / 4,
            }
            for target in ctx.targets.iter_mut() {
                target.decrease_hp(dmg);
            }
        } else if ctx.name == Self::SKILL_2 {
            match self.difficulty {
                Difficulty::Insane => dmg = atk * 25 / 2,
                Difficulty::Torment => dmg = atk * 25 / 2,
                Difficulty::Lunatic => dmg = atk * 25 / 2,
                _ => dmg = atk * 25 / 4,
            }

            assert_eq!(
                ctx.targets.len(),
                4,
                "Fire of Severity 2 Skill is not a target of 4 people"
            );

            ctx.targets[2].stats.hp.saturating_sub(dmg / 2);
            ctx.targets[3].stats.hp.saturating_sub(dmg / 4);
        }
    }
}

impl FiresofSeverity {
    const SKILL_1: &str = "Fires of Severity 1";
    const SKILL_2: &str = "Fires of Severity 2";
}
