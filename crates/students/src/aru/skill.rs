use core::{
    character::Character,
    damage::Damage,
    effect::{Effect, EffectTiming},
    simulator::Simulator,
    skill::{Region, Skill, SkillEffect, SkillEffectTarget, SkillHeader, SkillMeta, SkillType},
    state::{State, StateData},
    uid::Uid,
};
use std::sync::Weak;

use crate::aru::params::{self};

#[derive(Debug)]
pub struct ExSkill {
    header: SkillHeader,
}

impl ExSkill {
    const OWNER_OFFSET: usize = 1;
    const SKILL_TYPE: SkillType = SkillType::Ex;
    const RANGE: u16 = 200;

    pub fn new(
        owner: Uid,
        name: &'static str,
        skill_offset: usize,
        params: params::ExParams,
        sim: Weak<Simulator>,
    ) -> Self {
        let id = (owner, Self::OWNER_OFFSET);
        let effects = vec![
            SkillEffect::new(
                id,
                EffectTiming::Instant,
                SkillEffectTarget::Boss {
                    kind: Effect::Damage {
                        coef_num: params.coefficient.first_damage_num,
                        coef_den: 100,
                    },
                },
            ),
            SkillEffect::new(
                id,
                EffectTiming::Instant,
                SkillEffectTarget::Land {
                    kind: Effect::Damage {
                        coef_num: params.coefficient.second_damage_num,
                        coef_den: 100,
                    },
                    region: Region::Arc {
                        radius: Self::RANGE,
                        start_angle_degree: 0,
                        end_angle_degree: 360,
                    },
                },
            ),
        ];
        let header = SkillHeader::new(
            owner,
            Self::OWNER_OFFSET,
            name,
            skill_offset,
            Self::SKILL_TYPE,
            effects,
            params.cost,
            params.duration,
            params.frames,
            sim,
        );

        Self { header }
    }
}

impl SkillMeta for ExSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl Skill for ExSkill {
    fn apply(&self, mut state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State {
        debug_assert!(targets.len() == 1);

        let target_char = targets[0];

        let target = state
            .search_uid_mut(target_char.uid())
            .expect("target uid not found");

        let target_coor = target.coordinate();

        state
    }
}
