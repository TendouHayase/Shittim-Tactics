use std::sync::Arc;

use crate::{base::BaseStats, skill::Skill, uid::Uid};

pub trait Character {
    fn uid(&self) -> Uid;
    fn stats(&self) -> &BaseStats;
    fn skills(&self) -> &[Arc<dyn Skill>];
    fn lookup_skill(&self, idx: usize) -> Option<&dyn Skill> {
        Some(self.skills().get(idx)?.as_ref())
    }
}
