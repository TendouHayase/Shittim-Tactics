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
