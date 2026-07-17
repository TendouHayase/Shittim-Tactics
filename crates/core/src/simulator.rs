use std::{collections::HashMap, sync::Arc};

use crate::{
    actions::ActionContext,
    damage::{Damage, key::SkillsBitMask},
    skill::Skill,
    state::Stateful,
};

pub trait Simulator {
    type S<'a>: Stateful<'a>;
    fn legal_actions<'a>(&self, state: &impl Stateful<'a>) -> Vec<Arc<dyn Skill>>;
    fn apply<'a: 'b, 'b, 'c>(
        &self,
        state: &'b Self::S<'a>,
        action: &'b ActionContext<dyn Skill + 'c>,
    ) -> Self::S<'a>;
    fn advance<'a: 'b, 'b>(
        &self,
        state: &'b Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error>;
    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16;
    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage>;
    fn is_time_over(&self, ticks: u16) -> bool;
}
