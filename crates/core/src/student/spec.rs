use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{table::gear::GearKind, uid::Uid};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub uid: Uid,
    pub name: String,

    pub level: u8,
    pub star: u8,
    pub weapon_star: u8,

    /// The elements in this array represent the levels of the following skills.
    /// Ex skill, Basic Skill, Enhanced Skill, Sub Skill
    pub skill_levels: [u8; 4],
    pub weapon_level: u8,
    pub bond_level: u8,

    /// Affinity Level of the Separated Character
    pub alter_bond_levels: Vec<u8>,

    pub gear_kinds: [GearKind; 3],
    pub gear_tiers: [u8; 3],
    pub gear_levels: [u8; 3],

    /// Each element in this array represents the following.
    /// Max HP Talent level, ATK Talent Level, Healing Talent Level
    pub talent_levels: [u8; 3],

    pub unique_item_level: Option<u8>,
}
