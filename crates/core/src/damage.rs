use std::ops::{Div, Mul};

use stochastic::{dist::Uniform, pmf::Pmf};

use crate::{
    base::BaseStats,
    damage::utils::{apply_def, crit_rate_fraction, stability_coefficient},
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

    /// A hit that always deals exactly `value`.
    pub fn point(value: u64) -> Self {
        let roll = Uniform {
            min: value,
            max: value,
        };

        Self {
            normal: roll,
            crit: roll,
            crit_num: 0,
            crit_den: 1,
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

    /// `src` and `tgt` are expected to already include their buffs and debuffs.
    pub fn attack_damage(
        src: &BaseStats,
        tgt: &BaseStats,
        scale_num: u64,
        scale_den: u64,
    ) -> Damage {
        let mut effectiveness_dmg_scale: u32 = 0;
        if is_weak(src.attack_type, tgt.armor_type) {
            match src.attack_type {
                AttackType::Explosive => effectiveness_dmg_scale = src.explosive_effectiveness,
                AttackType::Piercing => effectiveness_dmg_scale = src.piercing_effectiveness,
                AttackType::Mystic => effectiveness_dmg_scale = src.mystic_effectiveness,
                AttackType::Corrosive => effectiveness_dmg_scale = src.corrosive_effectiveness,
                AttackType::Sonic => effectiveness_dmg_scale = src.sonic_effectiveness,
                _ => (),
            }
        }

        effectiveness_dmg_scale += damage_scale(src.attack_type, tgt.armor_type);

        let mut max_dmg: u64 = src.atk.into();

        max_dmg *= effectiveness_dmg_scale as u64;

        max_dmg = apply_def(max_dmg, tgt.def, src.defense_piercing);

        // apply_def가 먼저 나눈 뒤 곱해 절사가 생기므로, 배율의 만분율 환산은 그 뒤에 한다.
        max_dmg /= 10000;

        max_dmg = max_dmg * src.dmg_dealt as u64 / 10000;
        // 10000이 중립값이라 그보다 낮으면 데미지가 늘어난다.
        max_dmg = max_dmg * 20000u64.saturating_sub(tgt.dmg_resist as u64) / 10000;

        max_dmg *= scale_num;
        max_dmg /= scale_den;

        let min_dmg: u64 =
            (max_dmg as f64 * stability_coefficient(src.stability, src.stability_rate)) as u64;

        let crit = crit_rate_fraction(src.crit, tgt.crit_res);

        Damage {
            normal: Uniform {
                min: min_dmg,
                max: max_dmg,
            },
            crit: Uniform {
                min: (min_dmg as f64 * (src.crit_dmg as f64 / 10000.0)) as u64,
                max: (max_dmg as f64 * (src.crit_dmg as f64 / 10000.0)) as u64,
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

/// How a hit's rolls are turned into what accumulates on its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DamageMode {
    /// Keep the full range and crit split of every hit.
    #[default]
    Distribution,
    /// Collapse every hit to its expected value.
    Deterministic,
}

impl DamageMode {
    pub fn resolve(self, damage: Damage) -> Damage {
        match self {
            DamageMode::Distribution => damage,
            DamageMode::Deterministic => Damage::point(damage.expected_value()),
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
