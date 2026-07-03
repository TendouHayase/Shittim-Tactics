use std::fmt::Debug;

use crate::skill::Skill;

pub trait Character: Debug + Clone {
    fn status(&self) -> &Self;
    fn decrease_hp(&mut self, amount: u64);
    fn walk(&mut self, x: f32, y: f32);
    fn skill_list(&self) -> &[Box<dyn Skill>];
}
