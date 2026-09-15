use error::Error;

use crate::skill::Skill;

pub trait Algorithm<'a> {
    fn search(&self, threshold: f64) -> Result<Vec<(&'a dyn Skill, u16)>, Error>;
}
