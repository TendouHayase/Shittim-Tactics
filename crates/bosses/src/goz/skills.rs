use crate::create_boss_skill;
use core::{
    skill::{Skill, SkillEffect, SkillType},
    state::{State, StateData},
    utils::time_to_ticks,
};

create_boss_skill!(GozNowYouSeeUs, 0, 0, 0, SkillType::Ex, 0, {
    fn skill_effects(&self) -> Vec<SkillEffect> {
        todo!()
    }

    fn apply<'b, 'c: 'b>(
        &self,
        _caster: &'c mut StateData,
        _targets: &'b mut [&'c mut StateData],
    ) {
        todo!()
    }
});

impl GozNowYouSeeUs {
    pub fn other_apply(_skill: &dyn Skill, _state: State) -> State {
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

        fn apply<'b, 'c: 'b>(
            &self,
            _caster: &'c mut StateData,
            _targets: &'b mut [&'c mut StateData],
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
            // match unsafe { self.parent.read().stats.difficulty } {
            //     Difficulty::Lunatic => {
            //         vec![SkillEffect {
            //             id: self.id,
            //             timing: EffectTiming::Instant,
            //             targets: vec![
            //                 SkillEffectTarget::Land {
            //                     kind: EffectKind::new_damage(),
            //                     region: core::skill::Region::Polygon {
            //                         vertex: [
            //                             (-260, -10000).into(),
            //                             (260, -10000).into(),
            //                             (-260, -140).into(),
            //                             (260, -140).into(),
            //                         ],
            //                         count: 4,
            //                     },
            //                 },
            //                 SkillEffectTarget::Land {
            //                     kind: EffectKind::new_buff(
            //                         BuffType::CostRecovery,
            //                         time_to_ticks(5, 1),
            //                         0,
            //                         350,
            //                     ),
            //                     region: core::skill::Region::Polygon {
            //                         vertex: [
            //                             (-260 + 520, -10000).into(),
            //                             (260 + 520, -10000).into(),
            //                             (-260 + 520, -140).into(),
            //                             (260 + 520, -140).into(),
            //                         ],
            //                         count: 4,
            //                     },
            //                 },
            //                 SkillEffectTarget::Land {
            //                     kind: EffectKind::new_buff(
            //                         BuffType::DmgDealt,
            //                         time_to_ticks(5, 1),
            //                         8, // 본래 7.5%지만 올림적용
            //                         350,
            //                     ),
            //                     region: core::skill::Region::Polygon {
            //                         vertex: [
            //                             (-260 - 520, -10000).into(),
            //                             (260 - 520, -10000).into(),
            //                             (-260 - 520, -140).into(),
            //                             (260 - 520, -140).into(),
            //                         ],
            //                         count: 4,
            //                     },
            //                 },
            //             ],
            //         }]
            //     }
            //     _ => vec![SkillEffect {
            //         id: self.id,
            //         timing: EffectTiming::Instant,
            //         targets: vec![
            //             SkillEffectTarget::Land {
            //                 kind: EffectKind::new_debuff(DebuffType::Stun, 5, 0, 0),
            //                 region: core::skill::Region::Polygon {
            //                     vertex: [
            //                         (-260 + 520, -10000).into(),
            //                         (260 + 520, -10000).into(),
            //                         (-260 + 520, -140).into(),
            //                         (260 + 520, -140).into(),
            //                     ],
            //                     count: 4,
            //                 },
            //             },
            //             SkillEffectTarget::Land {
            //                 kind: EffectKind::new_buff(
            //                     BuffType::CostRecovery,
            //                     time_to_ticks(5, 1),
            //                     0,
            //                     350,
            //                 ),
            //                 region: core::skill::Region::Polygon {
            //                     vertex: [
            //                         (-260 + 520, -10000).into(),
            //                         (260 + 520, -10000).into(),
            //                         (-260 + 520, -140).into(),
            //                         (260 + 520, -140).into(),
            //                     ],
            //                     count: 4,
            //                 },
            //             },
            //             SkillEffectTarget::Land {
            //                 kind: EffectKind::new_buff(
            //                     BuffType::DmgDealt,
            //                     time_to_ticks(5, 1),
            //                     8, // 본래 7.5%지만 올림적용
            //                     350,
            //                 ),
            //                 region: core::skill::Region::Polygon {
            //                     vertex: [
            //                         (-260 - 520, -10000).into(),
            //                         (260 - 520, -10000).into(),
            //                         (-260 - 520, -140).into(),
            //                         (260 - 520, -140).into(),
            //                     ],
            //                     count: 4,
            //                 },
            //             },
            //         ],
            //     }],
            // }
        }

        fn apply<'b, 'c: 'b>(
            &self,
            _caster: &'c mut StateData,
            _targets: &'b mut [&'c mut StateData],
        ) {
            todo!()
        }
    }
);
