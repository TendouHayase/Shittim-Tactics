use std::hash::{Hash, Hasher};

use crate::{
    damage::{Damage, key::SkillsBitMask},
    extra::ExtraStateData,
    uid::Uid,
    utils::Position,
};

pub trait Stateful: Clone + Send + Sync + Eq + Hash {
    fn new(students: &[StateData], boss: StateData, elased_frames: u16, cost: i8) -> Self;
    fn students(&self) -> &[StateData];
    fn students_mut(&mut self) -> &mut [StateData];
    fn boss(&self) -> &StateData;
    fn boss_mut(&mut self) -> &mut StateData;
    fn split_mut(&mut self) -> (&mut StateData, &mut [StateData]);
    fn cost(&self) -> i8;
    fn frames(&self) -> u16;
    fn state_data_by_uid(&self, uid: Uid) -> Option<&StateData>;
    fn state_data_by_uid_mut(&mut self, uid: Uid) -> Option<&mut StateData>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub students: StudentState,
    pub boss: StateData,
    pub frames: u16,
    pub cost: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StudentState {
    TotalAssault([StateData; 6]),
    FinalRestrictionRelease([StateData; 10]),
}

impl Stateful for State {
    fn new(students: &[StateData], boss: StateData, frames: u16, cost: i8) -> Self {
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

    fn students(&self) -> &[StateData] {
        match &self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    fn students_mut(&mut self) -> &mut [StateData] {
        match &mut self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    fn boss(&self) -> &StateData {
        &self.boss
    }

    fn boss_mut(&mut self) -> &mut StateData {
        &mut self.boss
    }

    fn split_mut(&mut self) -> (&mut StateData, &mut [StateData]) {
        let students = match &mut self.students {
            StudentState::TotalAssault(arr) => arr.as_mut_slice(),
            StudentState::FinalRestrictionRelease(arr) => arr.as_mut_slice(),
        };

        (&mut self.boss, students)
    }

    fn cost(&self) -> i8 {
        self.cost
    }

    fn frames(&self) -> u16 {
        self.frames
    }

    fn state_data_by_uid(&self, uid: Uid) -> Option<&StateData> {
        if uid == self.boss.common.uid {
            return Some(&self.boss);
        }

        self.students()
            .iter()
            .find(|&student| uid == student.common.uid)
            .map(|v| v as _)
    }

    fn state_data_by_uid_mut(&mut self, uid: Uid) -> Option<&mut StateData> {
        if uid == self.boss.common.uid {
            return Some(&mut self.boss);
        }

        for student in self.students_mut() {
            if uid == student.common.uid {
                return Some(student);
            }
        }

        None
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommonStateData {
    pub uid: Uid,

    pub cooldowns: Vec<u16>,
    pub remained_effects: Vec<RemainedEffects>,
    pub accumulated_damage: Vec<AccumulatedDamage>,
    pub effects: SkillsBitMask,

    pub coordinate: Position,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RemainedEffects {
    pub ticks: u16,
    pub offset: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccumulatedDamage {
    pub ticks: u16,
    pub damage: Option<Damage>,
}

impl PartialOrd for RemainedEffects {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RemainedEffects {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ticks.cmp(&other.ticks)
    }
}

#[derive(Debug, Clone)]
pub struct StateData {
    pub common: CommonStateData,
    pub extra: Option<Box<dyn ExtraStateData>>,
}

impl StateData {
    pub fn new(uid: Uid) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                coordinate: Default::default(),
                cooldowns: Vec::new(),
                effects: 0.into(),
                remained_effects: Vec::new(),
                accumulated_damage: Vec::new(),
            },
            extra: None,
        }
    }
}

impl StateData {
    pub fn from_parts(
        uid: Uid,
        coordinate: Position,
        cooldowns: &[u16],
        effects: SkillsBitMask,
        remained_effects: Vec<RemainedEffects>,
        accumulated_damage: &[AccumulatedDamage],
        extra: Box<dyn ExtraStateData>,
    ) -> Self {
        StateData {
            common: CommonStateData {
                uid,
                coordinate,
                accumulated_damage: accumulated_damage.to_vec(),
                cooldowns: cooldowns.to_vec(),
                effects: effects,
                remained_effects: remained_effects.clone(),
            },
            extra: Some(extra),
        }
    }

    pub fn clone_matching(
        &self,
        cooldowns_condition: impl Fn(&u16) -> u16,
        effects: SkillsBitMask,
        remained_effects: Vec<RemainedEffects>,
    ) -> Self {
        StateData {
            common: CommonStateData {
                uid: self.common.uid,
                coordinate: self.common.coordinate,

                cooldowns: self
                    .common
                    .cooldowns
                    .iter()
                    .map(cooldowns_condition)
                    .collect(),
                effects,
                remained_effects,
                accumulated_damage: self.common.accumulated_damage.clone(),
            },
            extra: self.extra.clone(),
        }
    }

    pub fn acc_damage(&self) -> Vec<Damage> {
        let mut result = Vec::with_capacity(self.common.accumulated_damage.len());
        for d in &self.common.accumulated_damage {
            if let Some(x) = d.damage {
                result.push(x)
            }
        }

        result
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
