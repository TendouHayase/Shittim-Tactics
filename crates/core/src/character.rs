use std::{fmt::Debug, rc::Rc};

use crate::{
    Position,
    base::BaseStats,
    skill::{Effect, Skill},
};

pub trait Character: Debug {
    fn id(&self) -> u32;
    fn stats(&self) -> &BaseStats;
    fn effects(&self) -> &Vec<Effect>;
    fn position(&self) -> &Position;
    fn decrease_hp(&mut self, amount: u64);
    fn walk(&mut self, x: f32, y: f32);
    fn skill_list(&self) -> Rc<Vec<Rc<dyn Skill>>>;
}
