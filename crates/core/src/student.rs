use std::{hash::Hash, rc::Rc};

use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{base::BaseStats, character::Character, skill::Skill};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StudentStats {
    pub student_stats: Box<StudentSpec>,
    pub base_stats: BaseStats,
}
#[derive(Debug, Clone)]
pub struct Student {
    pub stats: StudentStats,

    /// These are the student's Ex Skills, Basic Skills, Enhanced Skills, and Sub Skills.
    pub skills: Vec<Rc<dyn Skill>>,
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

impl Character for Student {
    fn id(&self) -> u32 {
        self.stats.student_stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skill_list(&self) -> &Vec<Rc<dyn Skill>> {
        &self.skills
    }
}

impl Hash for Student {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.hash(state);
    }
}

impl Eq for Student {}
