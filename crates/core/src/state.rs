use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use crate::{
    character::Character,
    damage::{Damage, cache::DamageCache, key::SkillsBitMask},
    utils::Position,
};

pub trait Stateful<'a, const MAX_EXTRA_SIZE: usize = 0>: Clone + Send + Sync + Eq + Hash {
    fn new(
        students: &[StateData<'a, MAX_EXTRA_SIZE>],
        boss: StateData<'a, MAX_EXTRA_SIZE>,
        elased_frames: u16,
        cost: i8,
    ) -> Self;
    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a, MAX_EXTRA_SIZE>];
    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a, MAX_EXTRA_SIZE>];
    fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a, MAX_EXTRA_SIZE>;
    fn boss_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut StateData<'a, MAX_EXTRA_SIZE>;
    fn cost(&self) -> i8;
    fn frames(&self) -> u16;
    fn is_terminated(&self) -> bool;
    fn is_goal(&self, threshold_percent: f64) -> bool;
    fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32)
    -> Option<&'c StateData<'a, MAX_EXTRA_SIZE>>;
    fn state_data_by_id_mut<'b: 'c, 'c>(
        &'b mut self,
        id: u32,
    ) -> Option<&'c mut StateData<'a, MAX_EXTRA_SIZE>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State<'a, const MAX_EXTRA_SIZE: usize> {
    pub students: StudentState<'a, MAX_EXTRA_SIZE>,
    pub boss: StateData<'a, MAX_EXTRA_SIZE>,
    pub frames: u16,
    pub cost: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StudentState<'a, const MAX_EXTRA_SIZE: usize> {
    TotalAssault([StateData<'a, MAX_EXTRA_SIZE>; 6]),
    FinalRestrictionRelease([StateData<'a, MAX_EXTRA_SIZE>; 10]),
}

impl<'a, const E: usize> Stateful<'a, E> for State<'a, E> {
    fn new(students: &[StateData<'a, E>], boss: StateData<'a, E>, frames: u16, cost: i8) -> Self {
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

    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a, E>] {
        match &self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a, E>]
    where
        'a: 'b,
        'b: 'c,
    {
        match &mut self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a, E> {
        &self.boss
    }

    fn boss_mut<'b, 'c>(&'b mut self) -> &'c mut StateData<'a, E>
    where
        'a: 'b,
        'b: 'c,
    {
        &mut self.boss
    }

    fn cost(&self) -> i8 {
        self.cost
    }

    fn frames(&self) -> u16 {
        self.frames
    }

    fn is_goal(&self, threshold_percent: f64) -> bool {
        self.boss
            .accumulated_damage_cache
            .get_or_compute(&self.boss.damage_list())
            .as_ref()
            .is_some_and(|x| x.query_range(0, self.boss.character.stats().hp) >= threshold_percent)
    }

    fn is_terminated(&self) -> bool {
        let mut result = true;

        for student in self.students() {
            if student
                .accumulated_damage_cache
                .get_or_compute(&student.damage_list())
                .as_ref()
                .is_some_and(|x| x.max < student.character.stats().hp)
            {
                result = false;
                break;
            }
        }

        result
    }

    fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32) -> Option<&'c StateData<'a, E>> {
        if id == self.boss.character.id() {
            return Some(&self.boss);
        }

        for student in self.students() {
            if id == student.character.id() {
                return Some(student);
            }
        }

        None
    }

    fn state_data_by_id_mut<'b: 'c, 'c>(&'b mut self, id: u32) -> Option<&'c mut StateData<'a, E>> {
        if id == self.boss.character.id() {
            return Some(&mut self.boss);
        }

        for student in self.students_mut() {
            if id == student.character.id() {
                return Some(student);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct StateData<'a, const EXTRA_BYTES: usize = 0> {
    pub cooldowns: Vec<u16>,
    pub remained_effects: BinaryHeap<Reverse<RemainedEffects>>,
    pub accumulated_damage: Vec<AccumulatedDamage>,

    pub damage_map: &'a HashMap<SkillsBitMask, Damage>,
    pub character: &'a Character,
    pub effects: SkillsBitMask,
    pub accumulated_damage_cache: DamageCache,

    pub coordinate: Position,

    pub extra: [u8; EXTRA_BYTES],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RemainedEffects {
    pub ticks: u16,
    pub bit: u8,
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

impl<const E: usize> PartialEq for StateData<'_, E> {
    fn eq(&self, other: &Self) -> bool {
        self.cooldowns == other.cooldowns
            && self.effects == other.effects
            && self.accumulated_damage == other.accumulated_damage
            && self.coordinate == other.coordinate
    }
}

impl<const E: usize> Eq for StateData<'_, E> {}

impl<const E: usize> Hash for StateData<'_, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (
            (self.character as *const Character) as *const usize as usize,
            &self.cooldowns,
            &self.effects,
            &self.accumulated_damage,
            self.coordinate,
        )
            .hash(state);
    }
}

impl<'a, const E: usize> StateData<'a, E> {
    pub fn with_extra<const NEW_E: usize>(self) -> StateData<'a, NEW_E> {
        let Self {
            cooldowns,
            remained_effects,
            accumulated_damage,
            damage_map,
            character,
            effects,
            accumulated_damage_cache,
            coordinate,
            extra: old_extra,
        } = self;

        let mut new_extra = [0u8; NEW_E];
        let copy_len = E.min(NEW_E);
        new_extra[..copy_len].copy_from_slice(&old_extra[..copy_len]);

        StateData {
            cooldowns,
            remained_effects,
            accumulated_damage,
            damage_map,
            character,
            effects,
            accumulated_damage_cache,
            coordinate,
            extra: new_extra,
        }
    }

    #[allow(unsafe_code)]
    pub fn with_zero_extra(self) -> StateData<'a> {
        self.with_extra::<0>()
    }

    #[allow(unsafe_code)]
    pub fn extra_as<T>(&self) -> &T {
        const { assert!(::std::mem::size_of::<T>() <= E) };
        unsafe { &*(self.extra.as_ptr() as *const T) }
    }

    #[allow(unsafe_code)]
    pub fn extra_as_mut<T>(&mut self) -> &mut T {
        const { assert!(::std::mem::size_of::<T>() <= E) };
        unsafe { &mut *(self.extra.as_mut_ptr() as *mut T) }
    }
}

impl<'a, const E: usize> StateData<'a, E> {
    pub fn new(character: &'a Character, skill_list: &'a HashMap<SkillsBitMask, Damage>) -> Self {
        StateData {
            character,
            coordinate: Default::default(),
            cooldowns: vec![0; character.skill_list().len()],
            effects: 0.into(),
            remained_effects: BinaryHeap::new(),
            accumulated_damage: Vec::new(),
            accumulated_damage_cache: Default::default(),
            damage_map: skill_list,
            extra: [0u8; E],
        }
    }

    pub fn from_parts<'b>(
        character: &'a Character,
        coordinate: Position,
        cooldowns: &[u16],
        effects: &'b SkillsBitMask,
        remained_effects: &'b BinaryHeap<Reverse<RemainedEffects>>,
        accumulated_damage: &'b [AccumulatedDamage],
        accumulated_damage_cache: DamageCache,
        skill_list: &'a HashMap<SkillsBitMask, Damage>,
    ) -> Self
    where
        'a: 'b,
    {
        StateData {
            character,
            coordinate,
            accumulated_damage_cache,
            cooldowns: cooldowns.to_vec(),
            effects: *effects,
            remained_effects: remained_effects.clone(),
            accumulated_damage: accumulated_damage.to_vec(),
            damage_map: skill_list,
            extra: [0u8; E],
        }
    }

    pub fn clone_matching(
        &self,
        cooldowns_condition: impl Fn(&u16) -> u16,
        effects: SkillsBitMask,
        remained_effects: BinaryHeap<Reverse<RemainedEffects>>,
    ) -> Self {
        StateData {
            character: self.character,
            coordinate: self.coordinate,
            accumulated_damage_cache: self.accumulated_damage_cache.clone(),
            cooldowns: self.cooldowns.iter().map(cooldowns_condition).collect(),
            effects,
            remained_effects,
            accumulated_damage: self.accumulated_damage.clone(),
            damage_map: self.damage_map,
            extra: [0u8; E],
        }
    }

    pub fn damage_list(&self) -> Vec<Damage> {
        let mut result = Vec::with_capacity(self.accumulated_damage.len());
        for d in &self.accumulated_damage {
            if let Some(x) = d.damage {
                result.push(x)
            }
        }

        result
    }

    pub fn damage_with_effects(&self) -> Option<Damage> {
        self.damage_map.get(&self.effects).copied()
    }
}
