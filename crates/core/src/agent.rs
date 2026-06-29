use crate::{
    actions::Action, character::Character, skill::Skill, state::State, student::StudentStat,
};

pub trait Agent {
    /// Returns the prior probability/score for each action in state s
    fn policy(&self, s: &impl State) -> &Vec<(Action<impl Skill>, f32)>;

    /// Estimating the Value of State s
    fn value(&self, s: &impl State) -> f32;
}
