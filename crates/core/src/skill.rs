use crate::character::Character;
use std::fmt::Debug;

pub trait Skill: Debug {
    /// Applies the skill to the target.
    fn apply(&self, target: &mut impl Character)
    where
        Self: Sized;
}
