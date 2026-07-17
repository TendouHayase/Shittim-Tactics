use std::{fmt::Debug, hash::Hash, rc::Rc, sync::Arc};

use crate::{base::BaseStats, skill::Skill};

pub trait Character: Debug + Send + Sync {
    fn id(&self) -> u32;
    fn stats(&self) -> &BaseStats;
    fn skill_list(&self) -> &Vec<Arc<dyn Skill>>;
}
