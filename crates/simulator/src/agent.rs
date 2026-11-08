use crate::{actions::ActionContext, simulator::Simulator, state::State};

pub trait Agent {
    type Value;

    fn value(&self, sim: &Simulator, state: &State) -> Self::Value;
    fn policy(&self, sim: &Simulator, state: &State) -> Vec<(ActionContext, f64)>;
}
