use crate::{actions::ActionContext, simulator::Simulator, state::Stateful};

pub trait Agent<'a, S: Stateful<'a>> {
    type Value;

    fn value(&self, sim: &impl Simulator<'a, S>, state: &S) -> Self::Value;
    fn policy(&self, sim: &impl Simulator<'a, S>, state: &S) -> Vec<(ActionContext<'a>, f64)>;
}
