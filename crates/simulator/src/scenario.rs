use core::{boss::Boss, damage::DamageMode, skill::Skill, student::Student, uid::SkillUid};
use std::sync::Arc;

pub struct Scenario {
    pub students: Vec<Student>,
    pub boss: Boss,
    pub skills: Vec<Arc<dyn Skill>>,
    pub limit_ticks: u16,
    pub damage_mode: DamageMode,
    pub initial_skill_queue: Vec<SkillUid>,
    pub min_input_interval: u16,
}
