//! Every pattern number comes from `data/bosses/binah.json`. The skills know nothing of
//! difficulty or json and see only the [`params`] they were built with.

use crate::create_boss_skill;
use core::{
    effect::{EffectKind, EffectTiming},
    skill::{SkillEffect, SkillEffectTarget, SkillKind, SkillMeta, SkillType},
    stat::StatKind,
    state::StateData,
};

/// The `Raw*` types that deserialize the json `skills` object, and the `*Params` that result
/// from picking one difficulty out of them.
pub mod params {
    use core::difficulty::{ByDifficulty, Difficulty};
    use core::locale::LocalizedName;
    use core::skill::{Region, SkillParams};
    use serde::Deserialize;

    /// Coefficients are all percentages, so the denominator is fixed.
    pub const PERCENT_DEN: u16 = 100;

    /// On-field capacity for this fight. How many actually stand there is a runtime fact, so
    /// this is only the bound `skill_effects` declares with; `apply` uses the targets it is
    /// given. Not `MAX_STUDENT_COUNT`, which is the party size of 10.
    pub const ON_FIELD_COUNT: u8 = 4;

    /// Keys of the json `skills` object are skill struct names without the boss prefix.
    #[derive(Debug, Deserialize)]
    pub struct RawSkills {
        #[serde(rename = "AtsilutsLight")]
        pub atsiluts_light: RawAtsilutsLight,

        #[serde(rename = "FiresofSeverity")]
        pub fires_of_severity: RawFiresOfSeverity,

        #[serde(rename = "PurifyingStorm")]
        pub purifying_storm: RawPurifyingStorm,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AtsilutsLightParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub instant_percent: u16,
        pub dot_percent: u16,
        pub dot_interval: u16,
        pub dot_duration: u16,
        /// Vertical rectangle the light covers, relative to the boss.
        pub region: Region,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawAtsilutsLight {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        instant_percent: ByDifficulty<u16>,
        dot_percent: ByDifficulty<u16>,
        dot_interval: ByDifficulty<u16>,
        dot_duration: ByDifficulty<u16>,
        region: ByDifficulty<Region>,
    }

    impl RawAtsilutsLight {
        pub fn pick(&self, difficulty: Difficulty) -> AtsilutsLightParams {
            AtsilutsLightParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                instant_percent: self.instant_percent[difficulty],
                dot_percent: self.dot_percent[difficulty],
                dot_interval: self.dot_interval[difficulty],
                dot_duration: self.dot_duration[difficulty],
                region: self.region[difficulty],
            }
        }
    }

    /// Fires twice at once: one hit on everyone, plus one on each of the four nearest strikers.
    #[derive(Debug, Clone, Copy)]
    pub struct FiresOfSeverityParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub all_percent: u16,
        /// Applied in order of distance from Binah. The count is fixed, so this is an ordered
        /// array rather than `(coefficient, count)` pairs.
        pub nearest_percents: [u16; 4],
    }

    #[derive(Debug, Deserialize)]
    pub struct RawFiresOfSeverity {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        all_percent: ByDifficulty<u16>,
        nearest_percents: ByDifficulty<[u16; 4]>,
    }

    impl RawFiresOfSeverity {
        pub fn pick(&self, difficulty: Difficulty) -> FiresOfSeverityParams {
            FiresOfSeverityParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                all_percent: self.all_percent[difficulty],
                nearest_percents: self.nearest_percents[difficulty],
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct PurifyingStormParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub percent: u16,
        pub def_down_scale: u16,
        pub def_down_duration: u16,
        pub count: u8,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawPurifyingStorm {
        pub name: LocalizedName,
        cost: ByDifficulty<u8>,
        duration: ByDifficulty<u16>,
        frames: ByDifficulty<u16>,
        percent: ByDifficulty<u16>,
        def_down_scale: ByDifficulty<u16>,
        def_down_duration: ByDifficulty<u16>,
        count: ByDifficulty<u8>,
    }

    impl RawPurifyingStorm {
        pub fn pick(&self, difficulty: Difficulty) -> PurifyingStormParams {
            PurifyingStormParams {
                cost: self.cost[difficulty],
                duration: self.duration[difficulty],
                frames: self.frames[difficulty],
                percent: self.percent[difficulty],
                def_down_scale: self.def_down_scale[difficulty],
                def_down_duration: self.def_down_duration[difficulty],
                count: self.count[difficulty],
            }
        }
    }

    impl SkillParams for AtsilutsLightParams {
        fn cost(&self) -> u8 {
            self.cost
        }

        fn duration(&self) -> u16 {
            self.duration
        }

        fn frames(&self) -> u16 {
            self.frames
        }
    }

    impl SkillParams for FiresOfSeverityParams {
        fn cost(&self) -> u8 {
            self.cost
        }

        fn duration(&self) -> u16 {
            self.duration
        }

        fn frames(&self) -> u16 {
            self.frames
        }
    }

    impl SkillParams for PurifyingStormParams {
        fn cost(&self) -> u8 {
            self.cost
        }

        fn duration(&self) -> u16 {
            self.duration
        }

        fn frames(&self) -> u16 {
            self.frames
        }
    }
}

fn damage_effect(percent: u16) -> EffectKind {
    EffectKind::Damage {
        coef_num: percent,
        coef_den: params::PERCENT_DEN,
    }
}

create_boss_skill!(
    BinahAtsilutsLight,
    params: params::AtsilutsLightParams,
    SkillType::Ex,
    SkillKind::Damage,
    0,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Land {
                        kind: damage_effect(params.instant_percent),
                        region: params.region,
                    }],
                },
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Persistent {
                        interval_frames: params.dot_interval,
                        duration_frames: params.dot_duration,
                    },
                    targets: vec![SkillEffectTarget::Land {
                        kind: damage_effect(params.dot_percent),
                        region: params.region,
                    }],
                },
            ]
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
    BinahFiresofSeverity,
    params: params::FiresOfSeverityParams,
    SkillType::Ex,
    SkillKind::Damage,
    1,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Student {
                        kind: damage_effect(params.all_percent),
                        count: params::ON_FIELD_COUNT,
                    }],
                },
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Instant,
                    targets: params
                        .nearest_percents
                        .iter()
                        .map(|&percent| SkillEffectTarget::Student {
                            kind: damage_effect(percent),
                            count: 1,
                        })
                        .collect(),
                },
            ]
        }

        fn apply(
            &self,
            _caster: &mut StateData,
            _targets: &mut [&mut StateData],
        ) {
        }
    }
);

create_boss_skill!(
    BinahPurifyingStorm,
    params: params::PurifyingStormParams,
    SkillType::Ex,
    SkillKind::Damage,
    2,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Student {
                        kind: EffectKind::Debuff {
                            ty: StatKind::Def,
                            duration: params.def_down_duration,
                            scale: params.def_down_scale,
                            amount: 0,
                        },
                        count: params.count,
                    }],
                },
                SkillEffect {
                    id: self.id(),
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Student {
                        kind: damage_effect(params.percent),
                        count: params.count,
                    }],
                },
            ]
        }

        fn apply(
            &self,
            _caster: &mut StateData,
            _targets: &mut [&mut StateData],
        ) {
        }
    }
);
