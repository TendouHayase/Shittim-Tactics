use std::{
    any::type_name,
    ops::{Add, AddAssign},
    sync::{Arc, RwLock},
};

use distrs::Normal as distrsNormal;
use num_traits::ToPrimitive;

use crate::{composite::Composite, range::RangeProbability, utils::build_prefix_sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Uniform {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug, Clone)]
pub struct IrwinHall {
    pub prefix_sum: Arc<Vec<u128>>,
    pub uniforms: Arc<RwLock<Vec<Uniform>>>,
    pub n: u32,
    pub min: u64,
    pub max: u64,
    pub total_combinations: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Normal {
    pub avg: f64,
    pub var: f64,
}

impl Uniform {
    pub fn new(min: u64, max: u64) -> Self {
        Self { min, max }
    }
}

impl Normal {
    pub fn new(avg: f64, dev: f64) -> Self {
        Self {
            avg,
            var: dev.powf(2.0),
        }
    }
}

impl PartialEq for IrwinHall {
    fn eq(&self, other: &Self) -> bool {
        self.max == other.max
            && self.min == other.min
            && self.n == other.n
            && self.total_combinations == other.total_combinations
            && self.prefix_sum == other.prefix_sum
    }
}

impl Eq for IrwinHall {}

impl Default for IrwinHall {
    fn default() -> Self {
        Self {
            prefix_sum: Arc::new(Vec::new()),
            uniforms: Arc::new(RwLock::new(Vec::new())),
            n: 0,
            min: 0,
            max: 0,
            total_combinations: 0,
        }
    }
}

impl IrwinHall {
    pub fn from_uniform(uniform: Uniform, n: u32) -> Self {
        let domain = uniform.max - uniform.min + 1;
        Self {
            prefix_sum: Arc::new(vec![domain as u128; domain as usize]),
            uniforms: Arc::new(RwLock::new(vec![uniform])),
            n,
            min: uniform.min,
            max: uniform.max,
            total_combinations: domain as u128,
        }
    }

    pub fn build_pmf(distr: &[Uniform]) -> Arc<Vec<u128>> {
        assert!(!distr.is_empty(), "distr must not be empty");

        let first = &distr[0];
        let first_len = (first.max - first.min + 1) as usize;
        let mut prefix: Vec<u128> = build_prefix_sum(&vec![1u128; first_len]);

        for u in &distr[1..] {
            let new_counts = Self::convolve_via_prefix(&prefix, u.min, u.max);
            prefix = build_prefix_sum(&new_counts);
        }

        Arc::new(prefix)
    }

    fn convolve_via_prefix(old_prefix: &[u128], umin: u64, umax: u64) -> Vec<u128> {
        let length = (umax - umin + 1) as usize;
        let old_len = old_prefix.len() - 1;
        let new_len = old_len + length - 1;

        let mut new_counts = vec![0u128; new_len];
        for t in 0..new_len {
            let lo = t.saturating_sub(length - 1);
            let hi = t.min(old_len - 1);
            if lo <= hi {
                new_counts[t] = old_prefix[hi + 1] - old_prefix[lo];
            }
        }
        new_counts
    }

    pub fn modify_pmf(&mut self, uniform: Uniform) {
        let new_counts = Self::convolve_via_prefix(&self.prefix_sum, uniform.min, uniform.max);
        let new_prefix = build_prefix_sum(&new_counts);

        self.min += uniform.min;
        self.max += uniform.max;
        self.total_combinations *= (uniform.max - uniform.min + 1) as u128;
        self.n += 1;

        self.prefix_sum = Arc::new(new_prefix);

        let mut guard = self.uniforms.write().unwrap();
        guard.push(uniform);
    }

    pub fn query_range(&self, start: u64, end: u64) -> f64 {
        if end < self.min || start > self.max {
            return 0.0;
        }

        let clamped_start = start.max(self.min);
        let clamped_end = end.min(self.max);

        let lo_idx = (clamped_start - self.min) as usize;
        let hi_idx = (clamped_end - self.min) as usize;

        let sum = self.prefix_sum[hi_idx + 1] - self.prefix_sum[lo_idx];

        sum as f64 / self.total_combinations as f64
    }

    pub fn pmf_at(&self, value: u64) -> f64 {
        if value < self.min || value > self.max {
            return 0.0;
        }
        let idx = (value - self.min) as usize;
        let count = self.prefix_sum[idx + 1] - self.prefix_sum[idx];
        count as f64 / self.total_combinations as f64
    }

    pub fn normal_approx(&self) -> Normal {
        let guard = self.uniforms.read().unwrap();
        let mut mean = 0.0f64;
        let mut variance = 0.0f64;

        for u in guard.iter() {
            let a = u.min as f64;
            let b = u.max as f64;
            mean += (a + b) / 2.0;
            let width = b - a + 1.0;
            variance += (width * width - 1.0) / 12.0;
        }

        Normal::new(mean, variance)
    }
}

impl AddAssign<&Uniform> for IrwinHall {
    fn add_assign(&mut self, rhs: &Uniform) {
        *self = self.compose(rhs);
    }
}

impl AddAssign<&IrwinHall> for IrwinHall {
    fn add_assign(&mut self, rhs: &IrwinHall) {
        let rhs_uniforms = rhs.uniforms.read().unwrap().clone();

        for u in rhs_uniforms {
            *self += &u;
        }
    }
}

impl Add for &IrwinHall {
    type Output = IrwinHall;

    fn add(self, rhs: &IrwinHall) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl Composite for Normal {
    type Output = Normal;
    fn compose(&self, rhs: &Normal) -> Self::Output {
        Self {
            avg: self.avg + rhs.avg,
            var: self.var + rhs.var,
        }
    }
}

impl<'a> Composite for Uniform {
    type Output = IrwinHall;
    fn compose(&self, rhs: &Self) -> Self::Output {
        let mut result = IrwinHall {
            prefix_sum: Arc::new(vec![]),
            uniforms: Arc::new(RwLock::new(vec![*self, *rhs])),
            n: 1,
            max: self.max,
            min: self.min,
            total_combinations: (self.max - self.min + 1) as u128,
        };

        result.prefix_sum = IrwinHall::build_pmf(&result.uniforms.read().unwrap());
        result
    }
}

impl Composite for IrwinHall {
    type Output = Self;
    fn compose(&self, rhs: &Self) -> Self::Output {
        self + rhs
    }
}
impl<'a> Composite<Uniform> for IrwinHall {
    type Output = IrwinHall;

    fn compose(&self, rhs: &Uniform) -> Self::Output {
        let new_counts = IrwinHall::convolve_via_prefix(&self.prefix_sum, rhs.min, rhs.max);
        let new_prefix = build_prefix_sum(&new_counts);

        let new_min = self.min + rhs.min;
        let new_max = self.max + rhs.max;
        let new_total = self.total_combinations * (rhs.max - rhs.min + 1) as u128;
        let new_n = self.n + 1;

        let mut new_uniforms = self.uniforms.read().unwrap().clone();
        new_uniforms.push(Uniform {
            min: rhs.min,
            max: rhs.max,
        });

        IrwinHall {
            prefix_sum: Arc::new(new_prefix),
            uniforms: Arc::new(RwLock::new(new_uniforms)),
            n: new_n,
            min: new_min,
            max: new_max,
            total_combinations: new_total,
        }
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd> RangeProbability<K> for Uniform {
    fn range_probability(&self, a: K, b: K) -> f64 {
        if a > b {
            return 0.0;
        }

        let a_u64 = a
            .to_u64()
            .unwrap_or_else(|| panic!("convert {} to i64 failed", type_name::<K>()));
        let b_u64 = b
            .to_u64()
            .unwrap_or_else(|| panic!("convert {} to i64 failed", type_name::<K>()));

        if a_u64 > b_u64 {
            return 0.0;
        }

        let start = a_u64.max(self.min);
        let end = b_u64.min(self.max);

        if start > end {
            return 0.0;
        }

        let count = (end - start + 1) as f64;
        let total = (self.max - self.min + 1) as f64;
        count / total
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd> RangeProbability<K> for Normal {
    fn range_probability(&self, a: K, b: K) -> f64 {
        if a > b {
            return 0.0;
        }

        let a_f64 = a
            .to_f64()
            .unwrap_or_else(|| panic!("convert {} to f64 failed", type_name::<K>()));
        let b_f64 = b
            .to_f64()
            .unwrap_or_else(|| panic!("convert {} to f64 failed", type_name::<K>()));

        let a_cdf = distrsNormal::cdf(a_f64 as f64, self.avg, self.var.sqrt());
        let b_cdf = distrsNormal::cdf(b_f64 as f64, self.avg, self.var.sqrt());

        b_cdf - a_cdf
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd> RangeProbability<K> for IrwinHall {
    fn range_probability(&self, a: K, b: K) -> f64 {
        if a > b {
            return 0.0;
        }
        // N이 12이상이면 정규분포로 근사
        if self.n >= 12 {
            return self.normal_approx().range_probability(a, b);
        }

        let a_u64 = a
            .to_u64()
            .unwrap_or_else(|| panic!("convert {} to i64 failed", type_name::<K>()));
        let b_u64 = b
            .to_u64()
            .unwrap_or_else(|| panic!("convert {} to i64 failed", type_name::<K>()));

        if b_u64 < self.min || a_u64 > self.max {
            return 0.0;
        }

        let clamped_start = a_u64.max(self.min);
        let clamped_end = b_u64.min(self.max);

        let lo_idx = (clamped_start - self.min) as usize;
        let hi_idx = (clamped_end - self.min) as usize;

        let sum = self.prefix_sum[hi_idx + 1] - self.prefix_sum[lo_idx];

        sum as f64 / self.total_combinations as f64
    }
}
