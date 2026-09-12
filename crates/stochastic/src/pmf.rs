use ordered_float::OrderedFloat;

use crate::dist::Uniform;

pub const MAX_CELLS: usize = 1_250_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pmf {
    offset: u64,
    mass: Vec<OrderedFloat<f64>>,
}

impl Default for Pmf {
    fn default() -> Self {
        Self {
            offset: 0,
            mass: vec![OrderedFloat(1.0)],
        }
    }
}

impl Pmf {
    pub fn min(&self) -> u64 {
        self.offset
    }

    pub fn max(&self) -> u64 {
        self.offset + self.mass.len() as u64 - 1
    }

    /// 칸 수는 이 표현에만 있는 개념이라 밖으로 내보내지 않는다. 측정용.
    #[cfg(test)]
    pub fn cells(&self) -> usize {
        self.mass.len()
    }

    /// Convolve in one hit: a single roll taken from `crit` with probability
    /// `p` and from `normal` otherwise.
    ///
    /// Convolving with a uniform is a box filter, so this is O(support) per
    /// hit rather than O(support × width). The literal double loop would be
    /// ~1e16 operations for a single EX skill and is not an option.
    pub fn push(&mut self, normal: Uniform, crit: Uniform, p: f64) {
        let lo = normal.min.min(crit.min);
        let hi = normal.max.max(crit.max);

        // 한 점에 몰린 타(고정 데미지 기믹)는 합성곱이 아니라 평행이동이다.``
        if lo == hi {
            self.offset += lo;
            return;
        }

        let old_len = self.mass.len();
        let new_len = old_len + (hi - lo) as usize;
        assert!(
            new_len <= MAX_CELLS,
            "pmf support would need {new_len} cells (~{} GB), over the {MAX_CELLS} cell cap",
            new_len as f64 * 8.0 / 1e9,
        );

        // 제자리 누적합. mass[i]는 이 시점부터 Σ_{x<=i} mass[x]를 뜻한다.
        for i in 1..old_len {
            self.mass[i].0 += self.mass[i - 1].0;
        }

        let q = 1.0 - p;
        let (na, nb) = ((normal.min - lo) as usize, (normal.max - lo) as usize);
        let (ca, cb) = ((crit.min - lo) as usize, (crit.max - lo) as usize);
        let nw = (nb - na + 1) as f64;
        let cw = (cb - ca + 1) as f64;

        let mut out = vec![OrderedFloat(0.0); new_len];
        for (y, dst) in out.iter_mut().enumerate() {
            let mut acc = 0.0;
            if q != 0.0 {
                acc += q * self.window(y, na, nb, old_len) / nw;
            }
            if p != 0.0 {
                acc += p * self.window(y, ca, cb, old_len) / cw;
            }
            *dst = OrderedFloat(acc);
        }

        self.mass = out;
        self.offset += lo;
    }

    /// Σ src[x] over x in [y-b, y-a] ∩ [0, len), read off the inclusive prefix
    /// sums that `push` has left in `self.mass`.
    ///
    /// Differencing prefix sums cancels: a window holding a tiny fraction of
    /// the total mass loses digits proportional to that fraction. Tolerable
    /// here because tail queries sum many cells back up, but it is the reason
    /// this is a baseline and not the shipped path.
    fn window(&self, y: usize, a: usize, b: usize, len: usize) -> f64 {
        if y < a {
            return 0.0;
        }
        let hi = (y - a).min(len - 1);
        let lo = y.saturating_sub(b);
        if lo > hi {
            return 0.0;
        }
        let lower = if lo == 0 { 0.0 } else { self.mass[lo - 1].0 };
        self.mass[hi].0 - lower
    }

    /// P(S >= t)
    pub fn tail(&self, t: u64) -> f64 {
        if t <= self.min() {
            return 1.0;
        }
        if t > self.max() {
            return 0.0;
        }
        // 뒤에서부터 더한다. 꼬리 끝이 가장 작아서 앞에서 더하면 흡수된다.
        self.mass[(t - self.offset) as usize..]
            .iter()
            .rev()
            .map(|x| x.0)
            .sum()
    }

    /// P(S < t)
    pub fn cdf(&self, t: u64) -> f64 {
        1.0 - self.tail(t)
    }

    /// P(a <= S <= b)
    pub fn range(&self, a: u64, b: u64) -> f64 {
        if b < self.min() || a > self.max() || b < a {
            return 0.0;
        }
        let lo = a.max(self.min()) - self.offset;
        let hi = b.min(self.max()) - self.offset;
        self.mass[lo as usize..=hi as usize]
            .iter()
            .map(|x| x.0)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(min: u64, max: u64) -> Uniform {
        Uniform { min, max }
    }

    /// 2026-08 총력전 딜러 실측값. DAMAGE_MODEL.md §1.4.
    fn measured() -> (Uniform, Uniform, f64) {
        (u(4_251_702, 5_462_154), u(19_447_000, 24_982_662), 0.51888)
    }

    fn brute(hits: &[(Uniform, Uniform, f64)]) -> Vec<f64> {
        let mut acc = vec![1.0f64];
        let mut base = 0u64;
        for &(n, c, p) in hits {
            let lo = n.min.min(c.min);
            let hi = n.max.max(c.max);
            let mut out = vec![0.0; acc.len() + (hi - lo) as usize];
            for (x, &m) in acc.iter().enumerate() {
                for v in n.min..=n.max {
                    out[x + (v - lo) as usize] += m * (1.0 - p) / (n.max - n.min + 1) as f64;
                }
                for v in c.min..=c.max {
                    out[x + (v - lo) as usize] += m * p / (c.max - c.min + 1) as f64;
                }
            }
            acc = out;
            base += lo;
        }
        let _ = base;
        acc
    }

    #[test]
    fn matches_brute_force() {
        let hits = [
            (u(3, 7), u(10, 14), 0.3),
            (u(1, 2), u(5, 9), 0.75),
            (u(0, 4), u(4, 4), 0.0),
        ];
        let mut pmf = Pmf::default();
        for &(n, c, p) in &hits {
            pmf.push(n, c, p);
        }
        let expected = brute(&hits);
        assert_eq!(pmf.cells(), expected.len());
        for (i, &e) in expected.iter().enumerate() {
            assert!(
                (pmf.mass[i] - e).abs() < 1e-15,
                "cell {i}: {} vs {e}",
                pmf.mass[i]
            );
        }
    }

    #[test]
    fn mass_sums_to_one() {
        let (n, c, p) = measured();
        let mut pmf = Pmf::default();
        for _ in 0..4 {
            pmf.push(n, c, p);
        }
        // 누적합 차분의 상쇄로 총질량이 흘러내린다. 4타 8.3e7칸에서 이미 1e-10.
        let total: f64 = pmf.mass.iter().map(|x| x.0).sum();
        assert!((total - 1.0).abs() < 1e-9, "total = {total}");
        assert_eq!(pmf.min(), 4 * n.min);
        assert_eq!(pmf.max(), 4 * c.max);
    }

    #[test]
    fn point_hit_is_a_shift() {
        let mut pmf = Pmf::default();
        pmf.push(u(0, 9), u(0, 9), 0.5);
        let before = pmf.clone();
        pmf.push(u(100, 100), u(100, 100), 0.4);
        assert_eq!(pmf.min(), before.min() + 100);
        assert_eq!(pmf.mass, before.mass);
    }
}
