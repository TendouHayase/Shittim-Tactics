use std::{hash::Hash, rc::Rc};

use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{
    Position,
    base::BaseStats,
    character::Character,
    damage::{Damage, DamageCache},
    skill::{Effect, Skill},
};

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
    pub student_stats: Rc<StudentSpec>,
    pub base_stats: BaseStats,

    /// These are the student's coordinates.
    pub coordinate: Position,

    /// These are the cooldowns for EX, Basic, Enhanced, and Sub Skills.
    pub cooldowns: [u32; 4],

    pub effects: Vec<Effect>,
}
#[derive(Debug, Clone)]
pub struct Student {
    pub stats: StudentStats,

    pub accumulated_damage: Vec<Damage>,
    pub accumulated_damage_cache: DamageCache,
    /// These are the student's Ex Skills, Basic Skills, Enhanced Skills, and Sub Skills.
    pub skills: Rc<Vec<Rc<dyn Skill>>>,
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats && self.accumulated_damage == other.accumulated_damage
    }
}

impl Character for Student {
    fn id(&self) -> u32 {
        self.stats.student_stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn effects(&self) -> &Vec<Effect> {
        &self.stats.effects
    }

    fn position(&self) -> &Position {
        &self.stats.coordinate
    }

    fn decrease_hp(&mut self, amount: u64) {
        self.stats.base_stats.hp -= amount;
    }

    fn walk(&mut self, x: f32, y: f32) {
        self.stats.coordinate.x = ordered_float::OrderedFloat(x);
        self.stats.coordinate.y = ordered_float::OrderedFloat(y);
    }

    fn skill_list(&self) -> Rc<Vec<Rc<dyn Skill>>> {
        self.skills.clone()
    }
}

impl Hash for Student {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.hash(state);
    }
}

impl Eq for Student {}
