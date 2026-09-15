use error::Error;

use crate::{
    actions::ActionContext,
    character::Character,
    skill::Skill,
    state::State,
    uid::{SkillUid, Uid},
};

pub trait Simulator {
    fn initial_state(&self) -> State;

    fn legal_actions(&self, state: &State) -> Vec<ActionContext>;

    fn resolve_targets(&self, state: &State, skill: &dyn Skill) -> Vec<Uid>;

    fn apply(&self, state: State, action: &ActionContext) -> Result<State, Error>;

    /// Advances `state` by `delta_ticks`.
    fn advance(&self, state: &State, delta_ticks: u16) -> Result<State, Error>;

    /// Ticks from `state` until the next point where anyone can act.
    fn next_event_frames(&self, state: &State) -> u16;

    /// Whether `ticks` is past the time limit.
    fn is_time_over(&self, ticks: u16) -> bool;

    /// The skill at a given skill offset.
    fn lookup_skill(&self, uid: SkillUid) -> Option<&dyn Skill>;

    /// The character with this `id`, if there is one.
    fn character_by_uid(&self, id: Uid) -> Option<&dyn Character>;
}
