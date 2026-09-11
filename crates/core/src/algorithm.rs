use crate::skill::Skill;

pub trait Algorithm<'a> {
    fn search(&self, threshold: f64) -> Vec<(&'a dyn Skill, u16)>;
}
