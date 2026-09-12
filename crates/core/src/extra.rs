use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use error::Error;

pub trait ExtraState: Any + Debug + Clone + PartialEq + Eq + Hash + Send + Sync {}

pub trait ExtraStateData: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn ExtraStateData>;
    fn eq_dyn(&self, other: &dyn ExtraStateData) -> bool;
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn ExtraStateData {
    pub fn downcast_as<T: 'static + ExtraStateData>(&self) -> Result<&T, Error> {
        self.as_any()
            .downcast_ref::<T>()
            .ok_or(Error::WrongType(format!(
                "expected type: {}, found: {}",
                std::any::type_name::<T>(),
                self.type_name()
            )))
    }

    pub fn downcast_as_mut<T: 'static + ExtraStateData>(&mut self) -> Result<&mut T, Error> {
        let name = self.type_name();
        self.as_any_mut()
            .downcast_mut::<T>()
            .ok_or(Error::WrongType(format!(
                "expected type: {}, found: {}",
                std::any::type_name::<T>(),
                name
            )))
    }
}

impl<T: ExtraState> ExtraStateData for T
where
    T: Any + Debug + Clone + PartialEq + Eq + Hash + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn ExtraStateData> {
        Box::new(self.clone())
    }

    fn eq_dyn(&self, other: &dyn ExtraStateData) -> bool {
        other.as_any().downcast_ref::<Self>() == Some(self)
    }

    fn hash_dyn(&self, mut state: &mut dyn std::hash::Hasher) {
        TypeId::of::<Self>().hash(&mut state); // 타입도 같아야함
        Hash::hash(self, &mut state);
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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
