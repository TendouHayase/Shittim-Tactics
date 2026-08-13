//! Every pattern number comes from `data/bosses/binah.json`. The skills know nothing of
//! difficulty or json and see only the [`params`] they were built with.

use crate::create_boss_skill;
use core::{
    boss::Boss,
    character::{Character, CharacterOps},
    effect::EffectTiming,
    skill::{EffectKind, SkillEffect, SkillEffectTarget, SkillMeta, SkillOps, SkillType},
    stat::StatKind,
    state::{AccumulatedDamage, StateData},
};
use std::ptr::NonNull;

/// The `Raw*` types that deserialize the json `skills` object, and the `*Params` that result
/// from picking one difficulty out of them.
///
/// Kept inside a module: a top-level `struct` here would be mistaken for a skill by xtask and
/// pulled into the `Skill` enum.
pub mod params {
    use core::difficulty::{ByDifficulty, Difficulty};
    use core::locale::LocalizedName;
    use core::skill::Region;
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
}

fn damage_effect(percent: u16) -> EffectKind {
    EffectKind::Damage {
        coef_num: percent,
        coef_den: params::PERCENT_DEN,
    }
}

/// Zips `percents` onto targets from the front. Surplus values are dropped and surplus targets
/// are left alone.
///
/// Order is the target selection, so distance-ordered patterns rely on `targets` already being
/// sorted.
fn append_damage(
    caster: &StateData<'_>,
    targets: &mut [&mut StateData<'_>],
    percents: impl IntoIterator<Item = u16>,
    ticks: u16,
) {
    let Some(damage) = caster.damage_with_effects() else {
        return;
    };

    for (target, percent) in targets.iter_mut().zip(percents) {
        target
            .accumulated_damage_cache
            .append(&(damage * percent as u64 / params::PERCENT_DEN as u64));
        target.accumulated_damage.push(AccumulatedDamage {
            ticks,
            damage: target.damage_map.get(&target.effects).copied(),
        });
    }
}

create_boss_skill!(
    AtsilutsLight,
    params: params::AtsilutsLightParams,
    SkillType::Ex,
    0,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id,
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Land {
                        kind: damage_effect(params.instant_percent),
                        region: params.region,
                    }],
                },
                SkillEffect {
                    id: self.id,
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

        fn apply<'a: 'b, 'b, 'c: 'b>(
            &self,
            _caster: &'c mut StateData<'a>,
            _targets: &'b mut [&'c mut StateData<'a>],
        ) {
            todo!()
        }
    }
);

create_boss_skill!(
    FiresofSeverity,
    params: params::FiresOfSeverityParams,
    SkillType::Ex,
    1,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id,
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Student {
                        kind: damage_effect(params.all_percent),
                        count: params::ON_FIELD_COUNT,
                    }],
                },
                SkillEffect {
                    id: self.id,
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

        fn apply<'a: 'b, 'b, 'c: 'b>(
            &self,
            caster: &'c mut StateData<'a>,
            targets: &'b mut [&'c mut StateData<'a>],
        ) {
            let params = self.params;
            let ticks = self.duration();

            append_damage(caster, targets, std::iter::repeat(params.all_percent), ticks);
            append_damage(caster, targets, params.nearest_percents, ticks);
        }
    }
);

create_boss_skill!(
    PurifyingStorm,
    params: params::PurifyingStormParams,
    SkillType::Ex,
    2,
    {
        fn skill_effects(&self) -> Vec<SkillEffect> {
            let params = self.params;

            vec![
                SkillEffect {
                    id: self.id,
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
                    id: self.id,
                    timing: EffectTiming::Instant,
                    targets: vec![SkillEffectTarget::Student {
                        kind: damage_effect(params.percent),
                        count: params.count,
                    }],
                },
            ]
        }

        fn apply<'a: 'b, 'b, 'c: 'b>(
            &self,
            caster: &'c mut StateData<'a>,
            targets: &'b mut [&'c mut StateData<'a>],
        ) {
            let params = self.params;

            append_damage(
                caster,
                targets,
                std::iter::repeat_n(params.percent, params.count as usize),
                self.duration(),
            );
        }
    }
);
