use std::fmt::Debug;

use crate::skill::Skill;

pub trait Character: Debug + Sync + Send {
    fn status(&self) -> &Self
    where
        Self: Sized;
    fn decrease_hp(&mut self, amount: i32);
    fn walk(&mut self, x: f32, y: f32);
    fn skill_list(&self) -> Vec<Box<dyn Skill>>;
}
