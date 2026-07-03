use crate::{actions::Action, skill::Skill, state::State};

pub trait Simulator {
    type Sk: Skill;

    fn legal_actions(&self, state: &State) -> Vec<Action<Self::Sk>>;
    fn apply(&self, state: &State, action: &Action<dyn Skill>) -> State;
}
