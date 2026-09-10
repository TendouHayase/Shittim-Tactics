use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub trait ExtraStateData: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn ExtraStateData>;
    fn eq_dyn(&self, other: &dyn ExtraStateData) -> bool;
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn type_name(&self) -> &'static str;
}

impl Clone for Box<dyn ExtraStateData> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for dyn ExtraStateData {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn ExtraStateData {}
impl Hash for dyn ExtraStateData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}
