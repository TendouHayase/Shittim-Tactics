use num_traits::ToPrimitive;

pub trait RangeProbability<K: ToPrimitive + PartialEq + PartialOrd> {
    fn range_probability(&self, a: K, b: K) -> f64;
}
