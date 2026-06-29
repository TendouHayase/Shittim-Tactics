use crate::character::Character;

pub trait Skill {
    fn employ(&self, target: Box<dyn Character>);
}
