use std::{marker::PhantomData, sync::Arc};

use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{
    actions::Action::{self, Use},
    base::BaseStats,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Student {
    pub student_stats: StudentSpec,
    pub base_stats: BaseStats,

    /// The elements in this array represent the levels of the following skills.
    /// Ex skill, Basic Skill, Enhanced Skill, Sub Skill
    pub skill_levels: [u8; 4],
    pub weapon_level: u8,
    pub bond_level: u8,

    /// Affinity Level of the Separated Character
    pub alter_bond_levels: Vec<u8>,

    /// Each element in the array represents the tier of the equipment listed below.
    /// hat, gloves, shoes, bag, badge, hairpin, amulet, wristwatch, necklace, unique_item
    pub gear_tiers: [u8; 10],

    /// Each element in the array represents the level of the equipment listed below.
    /// hat, gloves, shoes, bag, badge, hairpin, amulet, wristwatch, necklace, unique_item
    pub gear_levels: [u8; 10],

    /// Each element in this array represents the following.
    /// Max HP Talent level, ATK Talent Level, Healing Talent Level
    pub talent_levels: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct StudentStat {
    stats: Arc<Student>,
    pub coordinate: (f32, f32),
}
