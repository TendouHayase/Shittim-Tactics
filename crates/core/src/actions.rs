use crate::skill::Skill;

pub enum Action<T: Skill + Sized> {
    Wait,
    Use(T),
}
