use crate::{actions::ActionContext, simulator::Simulator, state::Stateful};

pub trait Agent<S: Stateful> {
    type Value;

    fn value(&self, sim: &impl Simulator<S>, state: &S) -> Self::Value;
    fn policy<'s>(
        &self,
        sim: &'s impl Simulator<S>,
        state: &S,
    ) -> Vec<(ActionContext<'s>, f64)>;
}
