use std::hash::{Hash, Hasher};

use error::Error;

use crate::{
    damage::{Damage, DamageDist},
    effect::Effect,
    extra::ExtraStateData,
    uid::Uid,
    utils::Position,
};

/// A hit on the boss that has been cast but has not landed yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingHit {
    pub frame: u16,
    pub damage: Damage,
}
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommonStateData {
    pub uid: Uid,

    pub cooldowns: Vec<u16>,
    pub remained_effects: Vec<RemainedEffects>,
    pub hp: u64,
    pub shield: u64,
    // 현재 체력을 분포로 두면 0 클램프 때문에 합성곱이 깨지므로 누적 데미지를 저장한다.
    pub damage_taken: DamageDist,

    pub coordinate: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemainedEffects {
    pub end_frame: u16,
    pub effect: Effect,
    pub source: u8,
}

#[derive(Debug, Clone)]
pub struct StateData {
    pub(super) common: CommonStateData,
    extra: Option<Box<dyn ExtraStateData>>,
}

impl StateData {
    pub fn new(
        uid: Uid,
        hp: u64,
        skill_count: usize,
        extra: Option<Box<dyn ExtraStateData>>,
    ) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                hp,
                shield: 0,
                damage_taken: DamageDist::default(),
                coordinate: Default::default(),
                cooldowns: vec![0; skill_count],
                remained_effects: Vec::new(),
            },
            extra,
        }
    }

    pub fn from_parts(
        uid: Uid,
        hp: u64,
        shield: u64,
        coordinate: Position,
        cooldowns: &[u16],
        remained_effects: Vec<RemainedEffects>,
        extra: Option<Box<dyn ExtraStateData>>,
    ) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                coordinate,
                hp,
                shield,
                damage_taken: DamageDist::default(),
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

    pub const fn damage_taken(&self) -> &DamageDist {
        &self.common.damage_taken
    }

    pub const fn damage_taken_mut(&mut self) -> &mut DamageDist {
        &mut self.common.damage_taken
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
