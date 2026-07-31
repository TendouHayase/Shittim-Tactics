use crate::create_boss_skill;
use crate::{
    boss::Boss,
    character::Character,
    difficulty::Difficulty,
    skill::{
        BuffType, DebuffType, EffectKind, EffectTiming, Skill, SkillEffect, SkillEffectTarget,
        SkillOps, SkillType,
    },
    state::{State, StateData},
    utils::time_to_ticks,
};
use std::ptr::NonNull;
create_boss_skill!(GozNowYouSeeUs, 0, 0, 0, SkillType::Ex, 0, {
    fn skill_effects(&self) -> Vec<SkillEffect> {
        todo!()
    }
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    ) {
        todo!()
    }
});
impl GozNowYouSeeUs {
    pub fn other_apply<'a>(skill: &Skill, state: State<'a>) -> State<'a> {
        todo!()
    }
}
create_boss_skill!(
    GozThreeLightMonte,
    3,
    time_to_ticks(7, 1),
    time_to_ticks(16, 10),
    SkillType::Ex,
    1,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            todo!()
        }
        fn apply<'a: 'b, 'b, 'c: 'b>(
            &self,
            caster: &'c mut StateData<'a>,
            targets: &'b mut [&'c mut StateData<'a>],
        ) {
            todo!()
        }
    }
);
create_boss_skill!(
    GozMagicalCoinHat,
    0,
    time_to_ticks(7, 1),
    time_to_ticks(5, 1),
    SkillType::Ex,
    2,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            todo!()
        }
        fn apply<'a: 'b, 'b, 'c: 'b>(
            &self,
            caster: &'c mut StateData<'a>,
            targets: &'b mut [&'c mut StateData<'a>],
        ) {
            todo!()
        }
    }
);
