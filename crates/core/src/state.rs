use crate::{state::data::StateData, uid::Uid};

pub mod data;

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

#[allow(clippy::large_enum_variant)]
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
