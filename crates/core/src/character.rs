use crate::{base::BaseStats, skill::Skill, uid::Uid};

pub trait Character {
    fn id(&self) -> Uid;
    fn stats(&self) -> &BaseStats;
    fn skills(&self) -> &[Box<dyn Skill>];
}
