use core::{
    damage::Damage,
    difficulty::Difficulty,
    skill::{
        CasterContext, Debuff::Def, Effect, EffectKind, EffectTarget, EffectTiming, Skill,
        SkillType::Ex, TargetContext,
    },
};

#[derive(Debug)]
pub struct AtsilutsLight {
    difficulty: Difficulty,
}

impl Skill for AtsilutsLight {
    fn name(&self) -> &str {
        "Atsilut's Light"
    }
    fn cost(&self) -> u8 {
        0
    }

    fn frames(&self) -> u32 {
        1
    }

    fn effects(&self) -> Vec<Effect> {
        let duration: u32;

        match self.difficulty {
            Difficulty::Torment => duration = 15 * 30,
            Difficulty::Lunatic => duration = 120 * 30,
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
                    interval_frames: 90,
                    duration_frames: duration,
                },
                targets: vec![],
            },
        ]
    }

    fn apply(&self, caster: &mut CasterContext, target: &mut Vec<TargetContext>) {
        todo!()
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }
}

impl AtsilutsLight {
    const SKILL_1: &str = "Atsilut's Light 1";
    const SKILL_2: &str = "Atsilut's Light 2";

    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        AtsilutsLight { difficulty }
    }
}

#[derive(Debug)]
pub struct FiresofSeverity1 {
    difficulty: Difficulty,
}

impl Skill for FiresofSeverity1 {
    fn name(&self) -> &str {
        "Fire of Severity"
    }
    fn cost(&self) -> u8 {
        0
    }

    fn frames(&self) -> u32 {
        1
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            name: Self::NAME,
            kind: EffectKind::Damage,
            timing: EffectTiming::Instant,
            targets: vec![EffectTarget::Student; 4],
        }]
    }

    fn apply(&self, caster: &mut CasterContext, targets: &mut Vec<TargetContext>) {
        let dmg_num;
        let dmg_den;

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

        for target in targets.iter_mut() {
            let d = Damage::from_skill_context(caster, target, dmg_num, dmg_den);
            target.accumulated_damage.push(d);
            target.accumulated_damage_cached.append(&d);
        }
    }
}

impl FiresofSeverity1 {
    const NAME: &str = "Fire of Severity 1";

    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        FiresofSeverity1 { difficulty }
    }
}

#[derive(Debug)]
pub struct FireofSeverity2 {
    pub difficulty: Difficulty,
}

impl Skill for FireofSeverity2 {
    fn name(&self) -> &str {
        "Fire of Severity 2"
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn cost(&self) -> u8 {
        0
    }

    fn frames(&self) -> u32 {
        1
    }

    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            name: Self::NAME,
            kind: EffectKind::Damage,
            timing: EffectTiming::Instant,
            targets: vec![EffectTarget::Student; 4],
        }]
    }

    fn skill_type(&self) -> core::skill::SkillType {
        Ex
    }

    fn apply(&self, caster: &mut CasterContext, targets: &mut Vec<TargetContext>) {
        let dmg_num;
        let mut dmg_den;

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
            targets.len(),
            4,
            "Fire of Severity 2 Skill is not a target of 4 people"
        );

        for (i, target) in targets.iter_mut().enumerate() {
            let d = Damage::from_skill_context(caster, target, dmg_num, dmg_den);
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

impl FireofSeverity2 {
    const NAME: &str = "Fires of Severity 2";

    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        FireofSeverity2 { difficulty }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PurifyingStorm {
    pub difficulty: Difficulty,
}

impl Skill for PurifyingStorm {
    fn name(&self) -> &str {
        "Purifying Storm"
    }
    fn cost(&self) -> u8 {
        3
    }

    fn frames(&self) -> u32 {
        todo!()
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn effects(&self) -> Vec<Effect> {
        vec![Effect {
            name: "PurifyingStorm",
            kind: EffectKind::Debuff {
                ty: Def,
                duration: 90,
                scale: 50,
                amount: 0,
            },
            timing: EffectTiming::Instant,
            targets: vec![EffectTarget::Student; 4],
        }]
    }

    fn apply(&self, caster: &mut CasterContext, targets: &mut Vec<TargetContext>) {
        for target in targets.iter_mut() {
            let dmg = Damage::from_skill_context(caster, target, 300, 1);
            target.accumulated_damage.push(dmg);
            target.accumulated_damage_cached.append(&dmg);
        }
    }
}

impl PurifyingStorm {
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        PurifyingStorm { difficulty }
    }
}
