use std::ops::{Div, Mul};

use stochastic::{dist::Uniform, pmf::Pmf};

use crate::{
    base::BaseStats,
    character::Character,
    damage::utils::{apply_def, crit_rate, crit_rate_fraction, stability_coefficient},
    skill::Skill,
    state::StateData,
    types::{AttackType, damage_scale, is_weak},
};

pub mod utils;

/// A damage distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Damage {
    pub normal: Uniform,
    pub crit: Uniform,
    pub crit_num: u32,
    pub crit_den: u32,
}

impl Default for Damage {
    fn default() -> Self {
        Self {
            normal: Uniform { min: 0, max: 0 },
            crit: Uniform { min: 0, max: 0 },
            crit_num: 0,
            crit_den: 1,
        }
    }
}

impl Damage {
    pub fn new(
        normal_max: u64,
        normal_min: u64,
        crit_max: u64,
        crit_min: u64,
        crit_num: u32,
        crit_den: u32,
    ) -> Self {
        Self {
            normal: Uniform {
                min: normal_min,
                max: normal_max,
            },
            crit: Uniform {
                min: crit_min,
                max: crit_max,
            },
            crit_num,
            crit_den,
        }
    }

    pub fn expected_value(&self) -> u64 {
        let normal_avg = (self.normal.max + self.normal.min) / 2;
        let crit_avg = (self.crit.max + self.crit.min) / 2;

        (normal_avg * (self.crit_den - self.crit_num) as u64 + crit_avg * self.crit_num as u64)
            / self.crit_den as u64
    }

    pub fn crit_rate(&self) -> f64 {
        self.crit_num as f64 / self.crit_den as f64
    }

    pub fn attack_damage(
        src: &dyn Character,
        tgt: &dyn Character,
        scale_num: u64,
        scale_den: u64,
    ) -> Damage {
        // Since stats are added via multiplication or addition depending on buffs and debuffs,
        // a `scale` variable is required, so each element is copied rather than copying the entire object.
        let mut effectiveness_dmg_scale: u32 = 0;
        if is_weak(src.stats().attack_type, tgt.stats().armor_type) {
            match src.stats().attack_type {
                AttackType::Explosive => {
                    effectiveness_dmg_scale = src.stats().explosive_effectiveness
                }
                AttackType::Piercing => {
                    effectiveness_dmg_scale = src.stats().piercing_effectiveness
                }
                AttackType::Mystic => effectiveness_dmg_scale = src.stats().mystic_effectiveness,
                AttackType::Corrosive => {
                    effectiveness_dmg_scale = src.stats().corrosive_effectiveness
                }
                AttackType::Sonic => effectiveness_dmg_scale = src.stats().sonic_effectiveness,
                _ => (),
            }
        }

        effectiveness_dmg_scale = effectiveness_dmg_scale / 10000
            + damage_scale(src.stats().attack_type, tgt.stats().armor_type);

        let mut max_dmg: u64 = src.stats().atk.into();

        max_dmg *= effectiveness_dmg_scale as u64;

        max_dmg = apply_def(max_dmg, tgt.stats().def, src.stats().defense_piercing);

        max_dmg *= src.stats().dmg_dealt as u64 / 10000;
        max_dmg -= max_dmg * (tgt.stats().dmg_resist as u64 - 10000) / 10000;

        max_dmg *= scale_num;
        max_dmg /= scale_den;

        let min_dmg: u64 = (max_dmg as f64
            * stability_coefficient(src.stats().stability, src.stats().stability_rate))
            as u64;

        let crit = crit_rate_fraction(src.stats().crit, tgt.stats().crit_res);

        Damage {
            normal: Uniform {
                min: min_dmg,
                max: max_dmg,
            },
            crit: Uniform {
                min: (min_dmg as f64 * (src.stats().crit_dmg as f64 / 10000.0)) as u64,
                max: (max_dmg as f64 * (src.stats().crit_dmg as f64 / 10000.0)) as u64,
            },
            crit_num: crit.0,
            crit_den: crit.1,
        }
    }
}

impl Mul<u64> for Damage {
    type Output = Damage;
    fn mul(self, rhs: u64) -> Self::Output {
        Damage {
            normal: Uniform {
                min: self.normal.min * rhs,
                max: self.normal.max * rhs,
            },
            crit: Uniform {
                min: self.crit.min * rhs,
                max: self.crit.max * rhs,
            },
            crit_num: self.crit_num,
            crit_den: self.crit_den,
        }
    }
}

impl Div<u64> for Damage {
    type Output = Damage;
    fn div(self, rhs: u64) -> Self::Output {
        Damage {
            normal: Uniform {
                min: self.normal.min / rhs,
                max: self.normal.max / rhs,
            },
            crit: Uniform {
                min: self.crit.min / rhs,
                max: self.crit.max / rhs,
            },
            crit_num: self.crit_num,
            crit_den: self.crit_den,
        }
    }
}

/// Distribution of a sum of hits.
///
/// **This is the frozen boundary between the search and the probability code.**
/// The backing representation is an exact per-unit PMF today and becomes a
/// conditioned grid before release (see `.docs/DAMAGE_MODEL.md`); nothing
/// outside this module may observe which. Every signature here is therefore in
/// damage units and probabilities only. Adding methods is safe — changing or
/// widening these is what the swap must not have to do.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DamageDist {
    inner: Pmf,
}

impl DamageDist {
    pub fn push(&mut self, dmg: Damage) {
        self.inner.push(dmg.normal, dmg.crit, dmg.crit_rate());
    }

    /// Exact integer bound, never approximated by any backend. A\*'s
    /// admissibility rests on these two staying hard, so a backend swap may
    /// not turn them into estimates.
    pub fn min(&self) -> u64 {
        self.inner.min()
    }

    pub fn max(&self) -> u64 {
        self.inner.max()
    }

    /// P(S >= t)
    pub fn tail(&self, t: u64) -> f64 {
        self.inner.tail(t)
    }

    /// P(S < t)
    pub fn cdf(&self, t: u64) -> f64 {
        self.inner.cdf(t)
    }

    /// P(a <= S <= b)
    pub fn range(&self, a: u64, b: u64) -> f64 {
        self.inner.range(a, b)
    }
}
