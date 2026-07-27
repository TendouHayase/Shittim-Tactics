use crate::create_boss_skill;
use crate::{
    boss::Boss,
    character::Character,
    skill::{EffectKind, EffectTiming, Skill, SkillEffect, SkillEffectTarget, SkillType},
    state::{State, StateData},
    utils::time_to_ticks,
};
use std::ptr::NonNull;
create_boss_skill!(GozNowYouSeeUs, 0, 0, 0, SkillType::Ex, 0);
impl GozNowYouSeeUs {
    pub fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id,
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Oneself {
                kind: EffectKind::new_other(Self::other_apply),
            }],
        }]
    }
    pub fn apply<'a: 'b, 'b>(
        &self,
        caster: &'b mut StateData<'a>,
        targets: &'b mut [StateData<'a>],
    ) -> &'b mut [StateData<'a>] {
        todo!()
    }
    pub fn other_apply<'a>(skill: &Skill, state: State<'a>) -> State<'a> {
        state
    }
}
create_boss_skill!(
    GozThreeLightMonte,
    3,
    time_to_ticks(7, 1),
    time_to_ticks(16, 10),
    SkillType::Ex,
    1
);
impl GozThreeLightMonte {
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
        todo!()
    }
}
