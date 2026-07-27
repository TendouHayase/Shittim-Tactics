use crate::create_boss_skill;
use core::{
    boss::Boss,
    character::Character,
    skill::{EffectKind, EffectTiming, SkillEffect, SkillEffectTarget, SkillType},
    state::{State, StateData},
    utils::time_to_ticks,
};
use std::ptr::NonNull;

create_boss_skill!(NowYouSeeUs, 0, 0, 0, SkillType::Ex, 0);
impl NowYouSeeUs {
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Oneself {
                kind: EffectKind::new_other(Self::other_apply),
            }],
        }]
    }

    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
    }

    pub fn other_apply(skill: &Skill, state: State) -> State {
        state
    }
}
create_boss_skill!(
    ThreeLightMonte,
    3,
    time_to_ticks(7, 1),
    time_to_ticks(16, 10),
    SkillType::Ex,
    1
);
