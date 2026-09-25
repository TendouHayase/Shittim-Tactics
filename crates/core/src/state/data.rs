use std::hash::{Hash, Hasher};

use error::Error;

use crate::{
    damage::{Damage, DamageDist},
    effect::Effect,
    extra::ExtraStateData,
    uid::Uid,
    utils::Position,
};
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommonStateData {
    pub uid: Uid,

    pub cooldowns: Vec<u16>,
    pub remained_effects: Vec<RemainedEffects>,
    pub accumulated_damage: DamageDist,

    pub coordinate: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemainedEffects {
    pub ticks: u16,
    pub effect: Effect,
    pub source: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccumulatedDamage {
    pub ticks: u16,
    pub damage: Option<Damage>,
}

#[derive(Debug, Clone)]
pub struct StateData {
    pub(super) common: CommonStateData,
    extra: Option<Box<dyn ExtraStateData>>,
}

impl StateData {
    pub fn new(uid: Uid, cooldowns: usize, extra: Option<Box<dyn ExtraStateData>>) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                coordinate: Default::default(),
                cooldowns: vec![0; cooldowns],
                remained_effects: Vec::new(),
                accumulated_damage: Default::default(),
            },
            extra,
        }
    }

    pub fn from_parts(
        uid: Uid,
        coordinate: Position,
        cooldowns: &[u16],
        remained_effects: Vec<RemainedEffects>,
        accumulated_damage: DamageDist,
        extra: Option<Box<dyn ExtraStateData>>,
    ) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                coordinate,
                accumulated_damage,
                cooldowns: cooldowns.to_vec(),
                remained_effects: remained_effects.clone(),
            },
            extra,
        }
    }

    pub const fn uid(&self) -> Uid {
        self.common.uid
    }

    pub const fn uid_mut(&mut self) -> &mut Uid {
        &mut self.common.uid
    }

    pub const fn coordinate(&self) -> Position {
        self.common.coordinate
    }

    pub const fn coordinate_mut(&mut self) -> &mut Position {
        &mut self.common.coordinate
    }

    pub fn cooldowns(&self) -> &[u16] {
        &self.common.cooldowns
    }

    pub fn cooldowns_mut(&mut self) -> &mut [u16] {
        &mut self.common.cooldowns
    }

    pub fn remained_effects(&self) -> &[RemainedEffects] {
        &self.common.remained_effects
    }

    pub fn remained_effects_mut(&mut self) -> &mut Vec<RemainedEffects> {
        &mut self.common.remained_effects
    }

    pub fn accumulated_damage(&self) -> &DamageDist {
        &self.common.accumulated_damage
    }

    pub fn accumulated_damage_mut(&mut self) -> &mut DamageDist {
        &mut self.common.accumulated_damage
    }

    pub fn extra(&self) -> Option<&dyn ExtraStateData> {
        self.extra.as_deref()
    }

    pub fn extra_mut(&mut self) -> Option<&mut dyn ExtraStateData> {
        self.extra.as_deref_mut()
    }

    pub fn try_extra_as<T: ExtraStateData>(&self) -> Result<&T, Error> {
        self.extra().ok_or(Error::Empty)?.downcast_as::<T>()
    }

    pub fn try_extra_as_mut<T: ExtraStateData>(&mut self) -> Result<&mut T, Error> {
        self.extra_mut().ok_or(Error::Empty)?.downcast_as_mut::<T>()
    }

    pub fn extra_as<T: ExtraStateData>(&self) -> &T {
        self.try_extra_as().unwrap_or_else(|err| {
            panic!(
                "{:?} has no {} in extra: {err:?}",
                self.uid(),
                std::any::type_name::<T>()
            )
        })
    }

    pub fn extra_as_mut<T: ExtraStateData>(&mut self) -> &mut T {
        let uid = self.uid();
        self.try_extra_as_mut().unwrap_or_else(|err| {
            panic!(
                "{:?} has no {} in extra: {err:?}",
                uid,
                std::any::type_name::<T>()
            )
        })
    }
}

impl PartialEq for StateData {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common && self.extra == other.extra
    }
}

impl Eq for StateData {}

impl Hash for StateData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.common.hash(state);
        self.extra.hash(state);
    }
}
