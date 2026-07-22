use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use macros::unreachable_impl;

use crate::{
    character::Character,
    damage::{
        Damage,
        cache::DamageCache,
        key::{DamageKey, SkillsBitMask},
    },
    utils::Position,
};

#[macro_export]
macro_rules! create_state {
    ($boss_type:ty,  $($student_type:ty),+ $(,)?) => {
        #[repr(C, align(64))]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct State<'a> {
            pub students: ($(StateData<'a, $student_type>),+),
            pub boss: StateData<'a, $boss_type>,
            pub frames: u16,
            pub cost: i8,
        }

        impl<'a, const N: usize> Stateful<'a> for State<'a> {
            fn students<'b: 'c, 'c>(&'b self) -> &'b [StateData<'a>] {
                &self.students
            }

            fn students_mut<'b, 'c>(&'b mut self) -> &'c mut [StateData<'a>]
            where
                'a: 'b,
                'b: 'c,
            {
                &mut self.students
            }

            fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a> {
                &self.boss
            }

            fn boss_mut<'b, 'c>(&'b mut self) -> &'c mut StateData<'a>
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

                for student in &self.students {
                    if student
                        .accumulated_damage_cache
                        .get_or_compute(&self.boss.damage_list())
                        .as_ref()
                        .is_some_and(|x| x.max < student.character.stats().hp)
                    {
                        result = false;
                        break;
                    }
                }

                result
            }

            fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32) -> Option<&'c StateData<'a>> {
                if id == self.boss.character.id() {
                    return Some(&self.boss);
                }

                for student in &self.students {
                    if id == student.character.id() {
                        return Some(student);
                    }
                }

                None
            }
        }
    };
}

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct StateData<'a, T: Default + Clone + Send + Sync = ()> {
    pub cooldowns: Vec<u16>,
    pub remained_effects: BinaryHeap<Reverse<RemainedEffects>>,
    pub accumulated_damage: Vec<AccumulatedDamage<'a>>,

    pub character: &'a dyn Character,
    pub effects: DamageKey<'a>,
    pub accumulated_damage_cache: DamageCache,

    pub coordinate: Position,

    pub extras: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemainedEffects {
    pub ticks: u16,
    pub bit: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccumulatedDamage<'a> {
    pub damage: DamageKey<'a>,
    pub ticks: u16,
}

#[unreachable_impl]
pub trait Stateful<'a>: Clone + Send + Sync + Eq + Ord + Hash {
    fn new(students: &[StateData<'a>], boss: StateData<'a>, elased_frames: u16, cost: i8) -> Self;
    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a>];
    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a>];
    fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a>;
    fn boss_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut StateData<'a>;
    fn cost(&self) -> i8;
    fn frames(&self) -> u16;
    fn is_terminated(&self) -> bool;
    fn is_goal(&self, threshold_percent: f64) -> bool;
    fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32) -> Option<&'c StateData<'a>>;
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

impl PartialEq for StateData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cooldowns == other.cooldowns
            && self.effects == other.effects
            && self.accumulated_damage == other.accumulated_damage
            && self.coordinate == other.coordinate
    }
}

impl Eq for StateData<'_> {}

impl Hash for StateData<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (
            (self.character as *const dyn Character) as *const usize as usize,
            &self.cooldowns,
            &self.effects,
            &self.accumulated_damage,
            self.coordinate,
        )
            .hash(state);
    }
}

impl<'a> StateData<'a> {
    pub fn new(
        character: &'a dyn Character,
        skill_list: &'a HashMap<SkillsBitMask, Damage>,
    ) -> Self {
        StateData {
            character,
            coordinate: Default::default(),
            cooldowns: vec![0; character.skill_list().len()],
            effects: DamageKey::new(skill_list),
            remained_effects: BinaryHeap::new(),
            accumulated_damage: Vec::new(),
            accumulated_damage_cache: Default::default(),
            extras: Default::default(),
        }
    }

    pub fn from_parts<'b>(
        character: &'a dyn Character,
        coordinate: Position,
        cooldowns: &[u16],
        effects: &'b DamageKey<'a>,
        remained_effects: &'b BinaryHeap<Reverse<RemainedEffects>>,
        accumulated_damage: &'b [AccumulatedDamage<'a>],
        accumulated_damage_cache: DamageCache,
    ) -> Self
    where
        'a: 'b,
    {
        StateData {
            character,
            coordinate,
            accumulated_damage_cache,
            cooldowns: cooldowns.to_vec(),
            effects: effects.clone(),
            remained_effects: remained_effects.clone(),
            accumulated_damage: accumulated_damage.to_vec(),
            extras: Default::default(),
        }
    }

    pub fn clone_matching(
        &self,
        cooldowns_condition: impl Fn(&u16) -> u16,
        effects: DamageKey<'a>,
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
            extras: Default::default(),
        }
    }

    pub fn damage_list(&self) -> Vec<Damage> {
        let mut result = Vec::with_capacity(self.accumulated_damage.len());
        for d in &self.accumulated_damage {
            if let Some(x) = d.damage.damage() {
                result.push(x)
            }
        }

        result
    }
}
