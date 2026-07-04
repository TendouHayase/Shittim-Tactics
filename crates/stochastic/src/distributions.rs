use std::rc::Rc;

use crate::composite::Composite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UniformDistribution {
    pub min: u64,
    pub max: u64,
}

pub struct IrwinHallDistribution {
    pub head: UniformDistribution,
    pub prev: Option<Rc<IrwinHallDistribution>>,
    pub n: u32,
}

#[derive(Debug)]
pub struct NormalDistribution {
    pub avg: u64,
    pub var: u64,
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
