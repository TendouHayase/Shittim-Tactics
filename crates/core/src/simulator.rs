use crate::{actions::Action, skill::Skill, state::State};

pub trait Simulator {
    type S: State;
    type Sk: Skill;

    fn legal_actions(&self, state: &Self::S) -> Vec<Action<Self::Sk>>;
    fn apply(&self, state: &Self::S, action: &Action<Self::Sk>) -> Self::S;
}
