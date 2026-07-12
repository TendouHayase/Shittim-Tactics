use crate::{
    actions::ActionContext,
    boss::BossBehavior,
    skill::Skill,
    state::{State, Stateful},
};

pub trait Agent<T: BossBehavior + Clone> {
    /// Returns the prior probability/score for each action in state s
    fn policy<'a>(&self, s: &impl Stateful<'a>) -> &Vec<(ActionContext<impl Skill>, f32)>;

    /// Estimating the Value of State s
    fn value<'a>(&self, s: &impl Stateful<'a>) -> f32;
}
