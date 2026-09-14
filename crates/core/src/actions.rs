use crate::uid::{SkillUid, Uid};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionContext {
    pub caster: Uid,
    pub targets: Vec<Uid>,
    pub skill: SkillUid,
}
