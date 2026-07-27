use crate::create_boss_skill;
use crate::{
    boss::Boss,
    character::Character,
    skill::{SkillEffect, SkillType},
    state::StateData,
};
use std::ptr::NonNull;
create_boss_skill!(GozNowYouSeeUs, 0, 0, 0, SkillType::Ex, 0);
impl GozNowYouSeeUs {
    pub fn skill_effects(&self) -> Vec<SkillEffect> {}
    pub fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>> {
    }
}
