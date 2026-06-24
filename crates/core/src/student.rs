use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::base::BaseStats;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,

    #[builder(default = 2000)]
    pub stability_rate: u16,

    #[builder(default = 10000)]
    pub recovery_boost: u32,

    #[builder(default = 700)]
    pub cost_recovery: u16,

    #[builder(default = 10000)]
    pub atk_speed: u32,

    #[builder(default = 200)]
    pub mov_spd: u16,

    #[builder(default = 0)]
    pub block_rate_bonus: i16,

    pub mag_count: u8,

    #[builder(default = 10000)]
    pub dmg_dealt: u32,

    #[builder(default = 10000)]
    pub ex_skill_dmg_dealt: u32,

    #[builder(default = 10000)]
    pub ex_skill_dmg_resist: u32,

    #[builder(default = 10000)]
    pub basics_proficiency: u32,

    #[builder(default = 10000)]
    pub healing_boost: u32,

    #[builder(default = 10000)]
    pub buff_retention: u32,

    #[builder(default = 10000)]
    pub debuff_retention: u32,
}

pub struct Student {
    pub student_stats: Box<StudentSpec>,
    pub base_stats: Box<BaseStats>,

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
