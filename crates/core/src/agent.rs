use crate::{actions::ActionContext, simulator::Simulator, state::State};

pub trait Agent {
    type Value;

    fn value(&self, sim: &impl Simulator, state: &State) -> Self::Value;
    fn policy<'s>(&self, sim: &'s impl Simulator, state: &State) -> Vec<(ActionContext<'s>, f64)>;
}
