use std::collections::VecDeque;

use crate::{
    constants::{FINAL_RESTRICTION_RELEASE_HAND_SIZE, TOTAL_ASSAULT_HAND_SIZE},
    uid::{SkillUid, Uid},
};

pub mod data;
pub use data::*;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub students: StudentState,
    pub boss: StateData,
    pub frames: u16,
    /// Fixed point, see [`COST_UNIT`](crate::constants::COST_UNIT).
    pub cost: u32,
    /// Ex skills in play order. Only the front [`State::hand`] entries are usable.
    pub skill_queue: VecDeque<SkillUid>,
    /// Sorted by frame, ties in scheduling order.
    pub pending_hits: Vec<PendingHit>,
    /// Sorted by frame, ties in scheduling order.
    pub pending_casts: Vec<PendingCast>,
    /// The first frame the player can use another Ex skill.
    pub next_input_frame: u16,
}

impl State {
    pub fn new(
        students: &[StateData],
        boss: StateData,
        frames: u16,
        cost: u32,
        skill_queue: VecDeque<SkillUid>,
    ) -> Self {
        match students.len() {
            6 => Self {
                students: StudentState::TotalAssault(std::array::from_fn(|i| students[i].clone())),
                boss,
                frames,
                cost,
                skill_queue,
                pending_hits: Vec::new(),
                pending_casts: Vec::new(),
                next_input_frame: frames,
            },
            10 => Self {
                students: StudentState::FinalRestrictionRelease(std::array::from_fn(|i| {
                    students[i].clone()
                })),
                boss,
                frames,
                cost,
                skill_queue,
                pending_hits: Vec::new(),
                pending_casts: Vec::new(),
                next_input_frame: frames,
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

    pub const fn cost(&self) -> u32 {
        self.cost
    }

    pub const fn hand_size(&self) -> usize {
        match self.students {
            StudentState::TotalAssault(_) => TOTAL_ASSAULT_HAND_SIZE,
            StudentState::FinalRestrictionRelease(_) => FINAL_RESTRICTION_RELEASE_HAND_SIZE,
        }
    }

    pub fn hand(&self) -> impl Iterator<Item = SkillUid> + '_ {
        self.skill_queue.iter().copied().take(self.hand_size())
    }

    /// Sends a skill from the hand to the back of the queue. Returns whether it was in the hand.
    pub fn cycle_skill(&mut self, skill: SkillUid) -> bool {
        let Some(slot) = self.hand().position(|queued| queued == skill) else {
            return false;
        };

        self.skill_queue.remove(slot);
        self.skill_queue.push_back(skill);
        true
    }

    pub fn schedule_hit(&mut self, hit: PendingHit) {
        let slot = self
            .pending_hits
            .partition_point(|queued| queued.frame <= hit.frame);
        self.pending_hits.insert(slot, hit);
    }

    pub fn schedule_cast(&mut self, cast: PendingCast) {
        let slot = self
            .pending_casts
            .partition_point(|queued| queued.frame <= cast.frame);
        self.pending_casts.insert(slot, cast);
    }

    /// Removes and returns the earliest cast if it is due by `frame`.
    pub fn pop_cast_due(&mut self, frame: u16) -> Option<PendingCast> {
        let first = self.pending_casts.first()?;
        (first.frame <= frame).then(|| self.pending_casts.remove(0))
    }

    /// Removes and returns the first pending cast of `skill`.
    pub fn remove_cast(&mut self, skill: SkillUid) -> Option<PendingCast> {
        let slot = self
            .pending_casts
            .iter()
            .position(|cast| cast.action.skill == skill)?;
        Some(self.pending_casts.remove(slot))
    }

    /// Drops the hits of `source` due from `frame` on, for skills whose reuse replaces them.
    pub fn cancel_hits_from(&mut self, source: SkillUid, frame: u16) {
        self.pending_hits
            .retain(|hit| hit.source != source || hit.frame < frame);
    }

    /// Moves every hit due by the current frame onto the boss's damage distribution.
    pub fn settle_hits(&mut self) {
        let now = self.frames;
        let due = self.pending_hits.partition_point(|hit| hit.frame <= now);

        for hit in self.pending_hits.drain(..due) {
            self.boss.damage_taken_mut().push(hit.damage);

            if let Some(student) = self.students.search_uid_mut(hit.source.owner()) {
                *student.damage_dealt_mut() += hit.damage.expected_value();
            }
        }
    }

    /// The next frame a hit lands or a cast finishes. Hits are only known once their cast finishes,
    /// so both have to be stepped through to catch the frame the boss dies on.
    pub fn next_scheduled_frame(&self) -> Option<u16> {
        let hit = self.pending_hits.first().map(|hit| hit.frame);
        let cast = self.pending_casts.first().map(|cast| cast.frame);

        hit.into_iter().chain(cast).min()
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
