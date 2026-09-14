use std::hash::{Hash, Hasher};

use error::Error;

use crate::{
    damage::{Damage, DamageDist},
    extra::ExtraStateData,
    uid::Uid,
    utils::Position,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub students: StudentState,
    pub boss: StateData,
    pub frames: u16,
    pub cost: i8,
}

impl State {
    pub fn new(students: &[StateData], boss: StateData, frames: u16, cost: i8) -> Self {
        match students.len() {
            6 => Self {
                students: StudentState::TotalAssault(std::array::from_fn(|i| students[i].clone())),
                boss,
                frames,
                cost,
            },
            10 => Self {
                students: StudentState::FinalRestrictionRelease(std::array::from_fn(|i| {
                    students[i].clone()
                })),
                boss,
                frames,
                cost,
            },
            _ => panic!("unsupported students party size: {}", students.len()),
        }
    }

    pub const fn students(&self) -> &[StateData] {
        match &self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    pub const fn students_mut(&mut self) -> &mut [StateData] {
        match &mut self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    pub const fn boss(&self) -> &StateData {
        &self.boss
    }

    pub const fn boss_mut(&mut self) -> &mut StateData {
        &mut self.boss
    }

    pub fn split_mut(&mut self) -> (&mut StateData, &mut [StateData]) {
        let students = match &mut self.students {
            StudentState::TotalAssault(arr) => arr.as_mut_slice(),
            StudentState::FinalRestrictionRelease(arr) => arr.as_mut_slice(),
        };

        (&mut self.boss, students)
    }

    pub fn split_all_mut(&mut self) -> Vec<&mut StateData> {
        match &mut self.students {
            StudentState::TotalAssault(students) => {
                let [a, b, c, d, e, f] = students;
                vec![&mut self.boss, a, b, c, d, e, f]
            }
            StudentState::FinalRestrictionRelease(students) => {
                let [a, b, c, d, e, f, g, h, i, j] = students;
                vec![&mut self.boss, a, b, c, d, e, f, g, h, i, j]
            }
        }
    }

    pub const fn cost(&self) -> i8 {
        self.cost
    }

    pub const fn frames(&self) -> u16 {
        self.frames
    }

    pub fn search_uid(&self, uid: Uid) -> Option<&StateData> {
        if uid == self.boss.common.uid {
            return Some(&self.boss);
        }

        self.students()
            .iter()
            .find(|&student| uid == student.common.uid)
    }

    pub fn search_uid_mut(&mut self, uid: Uid) -> Option<&mut StateData> {
        if uid == self.boss.common.uid {
            return Some(&mut self.boss);
        }

        self.students_mut()
            .iter_mut()
            .find(|student| uid == student.common.uid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StudentState {
    TotalAssault([StateData; 6]),
    FinalRestrictionRelease([StateData; 10]),
}

impl StudentState {
    pub fn search_uid(&self, uid: Uid) -> Option<&StateData> {
        match self {
            StudentState::TotalAssault(arr) => {
                arr.iter().find(|&student| uid == student.common.uid)
            }
            StudentState::FinalRestrictionRelease(arr) => {
                arr.iter().find(|&student| uid == student.common.uid)
            }
        }
    }

    pub fn search_uid_mut(&mut self, uid: Uid) -> Option<&mut StateData> {
        match self {
            StudentState::TotalAssault(arr) => {
                arr.iter_mut().find(|student| uid == student.common.uid)
            }
            StudentState::FinalRestrictionRelease(arr) => {
                arr.iter_mut().find(|student| uid == student.common.uid)
            }
        }
    }
}

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
    pub effect: u8,
    pub source: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccumulatedDamage {
    pub ticks: u16,
    pub damage: Option<Damage>,
}

#[derive(Debug, Clone)]
pub struct StateData {
    common: CommonStateData,
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
