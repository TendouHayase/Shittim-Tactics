use std::rc::Rc;

#[derive(Debug)]
pub struct UniformDistribution {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug)]
pub struct IrwinHallDistributionElement<'a> {
    pub elem: UniformDistribution,
    pub prev: &'a UniformDistribution,
}

#[derive(Debug)]
pub struct IrwinHallDistribution<'a> {
    pub head: &'a IrwinHallDistributionElement<'a>,
    pub n: u32,
}

#[derive(Debug)]
pub struct NormalDistribution {
    pub avg: u64,
    pub var: u64,
}
