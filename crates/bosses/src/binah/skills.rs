use core::{
    damage::Damage,
    difficulty::Difficulty,
    skill::{
        Debuff::Def, EffectKind, EffectTiming, Skill, SkillEffect, SkillEffectTarget, SkillType::Ex,
    },
    state::{AccumulatedDamage, StateData},
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

    fn frames(&self) -> u16 {
        1
    }

    fn effects(&self) -> Vec<SkillEffect> {
        let duration: u16;

        match self.difficulty {
            Difficulty::Torment => duration = 15 * 30,
            Difficulty::Lunatic => duration = 120 * 30,
            _ => duration = 0,
        }

        vec![
            SkillEffect {
                name: Self::SKILL_1,
                kind: EffectKind::Damage,
                timing: EffectTiming::Instant,
                targets: vec![],
            },
            SkillEffect {
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

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b Vec<&'c StateData<'a>>,
    ) -> Vec<StateData<'a>> {
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

    fn frames(&self) -> u16 {
        1
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            name: Self::NAME,
            kind: EffectKind::Damage,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student; 4],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b Vec<&'c StateData<'a>>,
    ) -> Vec<StateData<'a>> {
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

        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for &target in targets.iter() {
            let d = caster.effects.damage();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * dmg_num / dmg_den));
                ac_dmg.push(AccumulatedDamage {
                    damage: target.effects.clone(),
                    ticks: self.frames(),
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
            ));
        }

        result
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

    fn frames(&self) -> u16 {
        1
    }

    fn effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            name: Self::NAME,
            kind: EffectKind::Damage,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student; 4],
        }]
    }

    fn skill_type(&self) -> core::skill::SkillType {
        Ex
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b Vec<&'c StateData<'a>>,
    ) -> Vec<StateData<'a>> {
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

        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for (i, &target) in targets.iter().enumerate() {
            let d = caster.effects.damage();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * dmg_num / dmg_den));
                ac_dmg.push(AccumulatedDamage {
                    damage: target.effects.clone(),
                    ticks: self.frames(),
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

    fn frames(&self) -> u16 {
        todo!()
    }

    fn owner(&self) -> &str {
        "Binah"
    }

    fn skill_type(&self) -> core::skill::SkillType {
        core::skill::SkillType::Ex
    }

    fn effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            name: "PurifyingStorm",
            kind: EffectKind::Debuff {
                ty: Def,
                duration: 90,
                scale: 50,
                amount: 0,
            },
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Student; 4],
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b Vec<&'c StateData<'a>>,
    ) -> Vec<StateData<'a>> {
        let mut result: Vec<StateData> = Vec::with_capacity(targets.len());

        for &target in targets.iter() {
            let d = caster.effects.damage();

            let mut ac_dmg = target.accumulated_damage.clone();
            let mut ac_dmg_cache = target.accumulated_damage_cache.clone();

            if let Some(damage) = d {
                ac_dmg_cache.append(&(damage * 3));
                ac_dmg.push(AccumulatedDamage {
                    damage: target.effects.clone(),
                    ticks: self.frames(),
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
            ));
        }

        result
    }
}

impl PurifyingStorm {
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        PurifyingStorm { difficulty }
    }
}
