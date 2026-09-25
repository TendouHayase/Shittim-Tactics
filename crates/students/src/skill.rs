use core::skill::{Skill, SkillParams};

pub trait ReadStudentSkill {
    fn ex_params() -> impl SkillParams;
    fn normal_params() -> impl SkillParams;
    fn sub_params() -> impl SkillParams;
}
