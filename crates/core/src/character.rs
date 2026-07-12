use std::{fmt::Debug, rc::Rc};

use crate::{base::BaseStats, skill::Skill};

pub trait Character: Debug {
    fn id(&self) -> u32;
    fn stats(&self) -> &BaseStats;
    fn skill_list(&self) -> &Vec<Rc<dyn Skill>>;
}
