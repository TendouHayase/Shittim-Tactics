use crate::create_boss_skill;
use core::{
    skill::{Skill, SkillEffect, SkillType},
    state::StateData,
};

/// Numbers not yet in json.
pub mod params {
    use core::utils::time_to_ticks;

    pub const THREE_LIGHT_MONTE_COST: u8 = 3;
    pub const THREE_LIGHT_MONTE_DURATION: u16 = time_to_ticks(7, 1);
    pub const THREE_LIGHT_MONTE_FRAMES: u16 = time_to_ticks(16, 10);

    pub const MAGICAL_COIN_HAT_DURATION: u16 = time_to_ticks(7, 1);
    pub const MAGICAL_COIN_HAT_FRAMES: u16 = time_to_ticks(5, 1);
}

create_boss_skill!(GozNowYouSeeUs, params: (), SkillType::Ex, 0, {
    fn skill_effects(&self) -> Vec<SkillEffect> {
        todo!()
    }

    fn apply(
        &self,
        _caster: &mut StateData,
        _targets: &mut [&mut StateData],
    ) {
        todo!()
    }
});

impl GozNowYouSeeUs {
    pub fn other_apply(
        _skill: &dyn Skill,
        _caster: &mut StateData,
        _targets: &mut [&mut StateData],
    ) {
        todo!()
    }
}

create_boss_skill!(
    GozThreeLightMonte,
    params: (),
    SkillType::Ex,
    1,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            todo!()
        }

        fn apply(
            &self,
            _caster: &mut StateData,
            _targets: &mut [&mut StateData],
        ) {
            todo!()
        }
    }
);

create_boss_skill!(
    GozMagicalCoinHat,
    params: (),
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

        fn apply(
            &self,
            _caster: &mut StateData,
            _targets: &mut [&mut StateData],
        ) {
            todo!()
        }
    }
);
