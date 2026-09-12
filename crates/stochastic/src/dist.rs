use crate::pmf::Pmf;

/// An inclusive damage range, rolled uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Uniform {
    pub min: u64,
    pub max: u64,
}

impl Uniform {
    pub fn new(min: u64, max: u64) -> Self {
        Self { min, max }
    }
}

/// One hit: a single roll, taken from `crit` with probability `p` and from
/// `normal` otherwise.
///
/// Crit scales the same roll rather than drawing again, which is why both
/// ranges carry the same stability ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub normal: Uniform,
    pub crit: Uniform,
    pub p: f64,
}

/// Distribution of a sum of hits.
///
/// **This is the frozen boundary between the search and the probability code.**
/// The backing representation is an exact per-unit PMF today and becomes a
/// conditioned grid before release (see `.docs/DAMAGE_MODEL.md`); nothing
/// outside this module may observe which. Every signature here is therefore in
/// damage units and probabilities only. Adding methods is safe — changing or
/// widening these is what the swap must not have to do.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HitBag {
    inner: Pmf,
}

impl HitBag {
    pub fn push(&mut self, hit: Hit) {
        self.inner.push(hit.normal, hit.crit, hit.p);
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
