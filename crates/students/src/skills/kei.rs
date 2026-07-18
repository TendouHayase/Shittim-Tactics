use core::{
    damage::key::DamageKey,
    skill::{BuffType, EffectKind, Region, Skill, SkillEffect, SkillType},
    state::{RemainedEffects, StateData},
    student::Student,
    types::AttackType,
    utils::is_inside,
};
use std::{cmp::Reverse, sync::Weak};

pub struct Kei {}

#[derive(Debug)]
pub struct KeiEx {
    kei: Weak<Student>,
    index: usize,
}

impl Skill for KeiEx {
    fn name(&self) -> &str {
        "Me, Here and Now"
    }

    fn cost(&self) -> u8 {
        2
    }

    fn duration(&self) -> u16 {
        123
    }

    fn owner(&self) -> std::sync::Weak<dyn core::character::Character> {
        self.kei.clone()
    }

    fn skill_mask_index(&self) -> usize {
        self.index
    }

    fn skill_type(&self) -> core::skill::SkillType {
        SkillType::Ex
    }

    fn skill_effects(&self) -> Vec<core::skill::SkillEffect> {
        vec![SkillEffect {
            name: self.name(),
            kind: EffectKind::Buff {
                ty: BuffType::Effectiveness(AttackType::Mystic),
                duration: 123,
                scale: (),
                amount: (),
            },
        }]
    }

    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b core::state::StateData<'a>,
        targets: &'b [&'c core::state::StateData<'a>],
    ) -> Vec<core::state::StateData<'a>> {
        let caster_coord = caster.coordinate;

        let mut result: Vec<StateData<'_>> = vec![];

        for &target in targets {
            if is_inside(target.coordinate, Self::REGION, caster_coord) {
                let already_applied = (target.effects.mask.0 & self.skill_mask_index() as u64) != 0;
                if already_applied {
                    result.push(target.clone());
                } else {
                    if target.character.id() == caster.character.id() {
                    } else {
                        let mut remained_effects = target.remained_effects.clone();
                        remained_effects.push(Reverse(RemainedEffects {
                            ticks: self.duration(),
                            bit: self.index as u8,
                        }));
                        result.push(StateData {
                            character: target.character,
                            coordinate: target.coordinate,
                            accumulated_damage_cache: target.accumulated_damage_cache.clone(),
                            cooldowns: target.cooldowns.clone(),
                            effects: DamageKey::from_mask(
                                (target.effects.mask.0 | (0x80u64 >> self.index)).into(),
                                &target.effects,
                            ),
                            remained_effects,
                            accumulated_damage: target.accumulated_damage.clone(),
                        });
                    }
                }
            }
        }

        result
    }
}

impl KeiEx {
    const REGION: Region = Region::Arc {
        radius: 1050,
        start_angle_degree: 0,
        end_angle_degree: 360,
    };
}
