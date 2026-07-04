pub trait Composite<Rhs = Self> {
    type Output;

    fn compose(&self, rhs: &Rhs) -> Self::Output;
}
