use num_traits::{Float, NumCast, ToPrimitive};

pub trait RangeProbability<K: ToPrimitive + PartialEq + PartialOrd, T: Float + NumCast> {
    fn range_probability(&self, a: K, b: K) -> Result<T, error::Error>;
}
