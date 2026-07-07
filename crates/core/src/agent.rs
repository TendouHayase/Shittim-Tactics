use crate::{actions::ActionContext, boss::BossTrait, skill::Skill, state::State};

pub trait Agent<T: BossTrait + Clone> {
    /// Returns the prior probability/score for each action in state s
    fn policy(&self, s: &State<T>) -> &Vec<(ActionContext<'_, impl Skill>, f32)>;

    /// Estimating the Value of State s
    fn value(&self, s: &State<T>) -> f32;
}
