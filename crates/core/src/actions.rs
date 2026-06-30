use std::rc::Rc;

use crate::skill::Skill;

#[derive(Debug)]
pub enum Action<T: Skill + ?Sized> {
    Wait,
    Use(Rc<T>),
}
