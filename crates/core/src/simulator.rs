use std::collections::HashMap;

use crate::{
    actions::ActionContext,
    character::Character,
    damage::{Damage, key::SkillsBitMask},
    skill::Skill,
    state::Stateful,
};

/// Implemented by whatever actually runs a simulation.
///
/// The `'a` in `S<'a>` must be the lifetime of the characters the simulation struct holds.
pub trait Simulator {
    type S<'a>: Stateful<'a>;

    /// Skills an agent may use from `state`.
    fn legal_actions<'a>(&self, state: &impl Stateful<'a>) -> Vec<&Skill>;

    /// Applies `action` to `state` and returns the resulting state.
    ///
    /// `action.targets` must not contain the caster; effects on the caster go through the
    /// `caster` argument of `Skill::apply`. Two mutable references to the same target cannot
    /// coexist, so a caster listed among the targets would simply be skipped.
    fn apply<'a>(&self, state: &Self::S<'a>, action: &ActionContext) -> Self::S<'a>;

    /// Advances `state` by `delta_ticks`.
    fn advance<'a>(
        &self,
        state: &Self::S<'a>,
        delta_ticks: u16,
    ) -> Result<Self::S<'a>, error::Error>;

    /// Ticks from `state` until the next point where anyone can act.
    fn next_event_frames<'a, 'b>(&self, state: &'b impl Stateful<'a>) -> u16;

    /// Damage keyed by which skills are active.
    fn damage_map(&self) -> &HashMap<SkillsBitMask, Damage>;

    /// Whether `ticks` is past the time limit.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// The skill at a given `SkillsBitMask` index.
    fn lookup_skill(&self, index: usize) -> Result<&Skill, error::Error>;

    /// The character with this `id`, if there is one.
    fn character_by_id(&self, id: u32) -> Option<Character<'_>>;
}
