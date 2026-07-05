use std::{any::type_name, rc::Rc};

use distrs::Normal;
use error::Error::{self, InvalidCasting};
use num_traits::{Float, Num, NumCast, ToPrimitive};

use crate::{composite::Composite, range::RangeProbability, utils::binomial};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UniformDistribution {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug)]
pub struct IrwinHallDistribution {
    pub head: UniformDistribution,
    pub prev: Option<Rc<IrwinHallDistribution>>,
    pub n: u32,
}

#[derive(Debug)]
pub struct NormalDistribution {
    pub avg: f64,
    pub var: f64,
}

impl Composite for NormalDistribution {
    type Output = NormalDistribution;
    fn compose(&self, rhs: &NormalDistribution) -> Self::Output {
        Self {
            avg: self.avg + rhs.avg,
            var: self.var + rhs.var,
        }
    }
}

impl<'a> Composite for UniformDistribution {
    type Output = IrwinHallDistribution;
    fn compose(&self, rhs: &Self) -> Self::Output {
        IrwinHallDistribution {
            head: self.clone(),
            prev: Some(Rc::new(IrwinHallDistribution {
                head: rhs.clone(),
                prev: None,
                n: 1,
            })),
            n: 1,
        }
    }
}

impl<'a> Composite<UniformDistribution> for Rc<IrwinHallDistribution> {
    type Output = Rc<IrwinHallDistribution>;
    fn compose(&self, rhs: &UniformDistribution) -> Self::Output {
        let mut result = IrwinHallDistribution {
            head: *rhs,
            prev: None,
            n: self.n + 1,
        };
        result.prev = Some(self.clone());
        Rc::from(result)
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd, T: Float + NumCast> RangeProbability<K, T>
    for UniformDistribution
{
    fn range_probability(&self, a: K, b: K) -> Result<T, error::Error> {
        if a > b {
            return Err(Error::InvalidArgument(
                "'a' must not be greater than 'b'".to_string(),
            ));
        }

        let a_u64 = a
            .to_u64()
            .expect(format!("convert {} to i64 failed", type_name::<K>()).as_str());
        let b_u64 = b
            .to_u64()
            .expect(format!("convert {} to i64 failed", type_name::<K>()).as_str());

        if b_u64 < self.min {
            return NumCast::from(0f64).ok_or(InvalidCasting(format!(
                "convert f64 to {} failed",
                type_name::<T>()
            )));
        }

        if a_u64 >= self.max {
            return NumCast::from(1f64).ok_or(InvalidCasting(format!(
                "convert f64 to {} failed",
                type_name::<T>()
            )));
        }

        let delta = (self.max - self.min) as f64;

        let a_cdf = if a_u64 < self.min {
            0f64
        } else {
            (a_u64 - self.min + 1) as f64 / delta
        };

        let b_cdf = if b_u64 >= self.max {
            1f64
        } else {
            (self.max - b_u64) as f64 / delta
        };

        Ok(NumCast::from(b_cdf - a_cdf)
            .expect(format!("convert f64 to {} failed", type_name::<T>()).as_str()))
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd, T: Float + NumCast> RangeProbability<K, T>
    for NormalDistribution
{
    fn range_probability(&self, a: K, b: K) -> Result<T, error::Error> {
        if a > b {
            return Err(Error::InvalidArgument(
                "'a' must not be greater than 'b'".to_string(),
            ));
        }

        let a_f64 = a
            .to_f64()
            .expect(format!("convert {} to f64 failed", type_name::<K>()).as_str());
        let b_f64 = b
            .to_f64()
            .expect(format!("convert {} to f64 failed", type_name::<K>()).as_str());

        let a_cdf = Normal::cdf(a_f64 as f64, self.avg, self.var.sqrt());
        let b_cdf = Normal::cdf(b_f64 as f64, self.avg, self.var.sqrt());

        Ok(NumCast::from(b_cdf - a_cdf)
            .expect(format!("convert f64 to {} failed", type_name::<T>()).as_str()))
    }
}

impl<K: ToPrimitive + PartialEq + PartialOrd, T: Float + NumCast> RangeProbability<K, T>
    for IrwinHallDistribution
{
    fn range_probability(&self, a: K, b: K) -> Result<T, error::Error> {
        // N이 30이상일때는 중심 극한 정리에 따라 정규분포에 근사되기에 정규분포를 이용
        if self.n >= 30 {
            let avg = self.n as f64 / 2.0f64;
            let var = (self.n as f64 / 12.0f64).sqrt();
            return NormalDistribution::range_probability(&NormalDistribution { avg, var }, a, b);
        };

        let a_u64 = a
            .to_u64()
            .expect(format!("convert {} to u64 failed", type_name::<K>()).as_str());
        let b_u64 = b
            .to_u64()
            .expect(format!("convert {} to u64 failed", type_name::<K>()).as_str());

        let mut a_cdf = 0;
        for i in 0..a_u64 {
            let sign: i64 = if i % 2 == 0 { 1 } else { -1 };
            let coeff: i64 = binomial(self.n.into(), i) as i64;
            let term: i64 = (a_u64 - i).pow(self.n) as i64;
            a_cdf += sign * coeff * term;
        }

        let mut b_cdf = 0;
        for i in 0..b_u64 {
            let sign: i64 = if i % 2 == 0 { 1 } else { -1 };
            let coeff: i64 = binomial(self.n.into(), i) as i64;
            let term: i64 = (a_u64 - i).pow(self.n) as i64;
            b_cdf += sign * coeff * term;
        }

        Ok(NumCast::from(b_cdf - a_cdf)
            .expect(format!("convert u64 to {} failed", type_name::<T>()).as_str()))
    }
}
