use core::{simulator::Simulator, skill::Skill, state::Stateful};

pub trait Algorithm {
    type S<'a>: Stateful<'a>;

    fn search<'a: 'b, 'b>(
        &'b self,
        simulator: &'a impl Simulator<S<'a> = Self::S<'a>>,
        initial: Self::S<'a>,
        threshold: f64,
    ) -> Vec<(&'b Skill, u16)>;
}
