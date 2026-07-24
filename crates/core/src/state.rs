use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use macros::unreachable_impl_for_empty;

use crate::{
    character::Character,
    damage::{Damage, cache::DamageCache, key::SkillsBitMask},
    utils::Position,
};

#[macro_export]
macro_rules! create_state {
    ($name:ident, $boss_extra_type:ty,  $($student_extra_type:ty),* $(,)?) => {
        paste! {
            const MAX_EXTRA_SIZE: usize = ::std::mem::size_of::<$boss_extra_type>()
                $( .max(::std::mem::size_of::<$student_extra_type>()) )*;

            #[repr(C)]
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct [<$name State>]<'a> {
                pub students: [StateData<'a, MAX_EXTRA_SIZE>; count($({$student_extra_type})*)],
                pub boss: StateData<'a, MAX_EXTRA_SIZE>,
                pub frames: u16,
                pub cost: i8,
            }
        }

        impl<'a, const N: usize> Stateful<'a> for [<$name State>]<'a> {
            fn new(students: &[StateData<'a>], boss: StateData<'a>, frames: u16, cost: i8) -> Self {
                let mut new_students: [StateData<'a, MAX_EXTRA_SIZE>; N] =
                    ::std::array::from_fn(|i| students[i].with_zero_extra());

                let mut _idx = 0usize;
                let extras = (
                    $({
                        let val: $student_extra_type = Default::default();
                        #[allow(unsafe_code)]
                        unsafe {
                            ::std::ptr::copy_nonoverlapping(
                                &val as *const $student_extra_type as *const u8,
                                new_students[_idx].extra.as_mut_ptr(),
                                ::std::mem::size_of::<$student_extra_type>(),
                            );
                        }
                        ::std::mem::forget(val);
                        _idx += 1;
                        $student_extra_type::default()
                    },)*
                );

                Self {
                    students: new_students,
                    boss: boss.with_zero_extra(),
                    frames,
                    cost,
                    extras,
                }
            }

            fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a>] {
                let ptr: *const StateData<'a, MAX_EXTRA_SIZE> = self.students.as_ptr();
                let len = self.students.len();
                #[allow(unsafe_code)]
                unsafe { ::std::slice::from_raw_parts(ptr as *const StateData<'a>, len) }
            }

            fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a>]
            where
                'a: 'b,
                'b: 'c,
            {
                let ptr: *mut StateData<'a, MAX_EXTRA_SIZE> = self.students.as_mut_ptr();
                let len = self.students.len();
                #[allow(unsafe_code)]
                unsafe { ::std::slice::from_raw_parts_mut(ptr as *mut StateData<'a>, len) }
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
                    .is_some_and(|x| {
                        x.query_range(0, self.boss.character.stats().hp) >= threshold_percent
                    })
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

            fn state_data_by_id_mut<'b: 'c, 'c>(&'b mut self, id: u32) -> Option<&'c mut StateData<'a>> {
                 if id == self.boss.character.id() {
                    return Some(&mut self.boss);
                }

                for student in &mut self.students {
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
pub struct StateData<'a, const EXTRA_BYTES: usize = 0> {
    pub cooldowns: Vec<u16>,
    pub remained_effects: BinaryHeap<Reverse<RemainedEffects>>,
    pub accumulated_damage: Vec<AccumulatedDamage>,

    pub damage_map: &'a HashMap<SkillsBitMask, Damage>,
    pub character: &'a dyn Character,
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

#[unreachable_impl_for_empty]
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
    fn state_data_by_id_mut<'b: 'c, 'c>(&'b mut self, id: u32) -> Option<&'c mut StateData<'a>>;
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
            (self.character as *const dyn Character) as *const usize as usize,
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
        debug_assert!(::std::mem::size_of::<T>() <= E);
        unsafe { &*(self.extra.as_ptr() as *const T) }
    }

    #[allow(unsafe_code)]
    pub fn extra_as_mut<T>(&mut self) -> &mut T {
        debug_assert!(::std::mem::size_of::<T>() <= E);
        unsafe { &mut *(self.extra.as_mut_ptr() as *mut T) }
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
            effects: 0.into(),
            remained_effects: BinaryHeap::new(),
            accumulated_damage: Vec::new(),
            accumulated_damage_cache: Default::default(),
            damage_map: skill_list,
            extra: [0u8; 0],
        }
    }

    pub fn from_parts<'b>(
        character: &'a dyn Character,
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
            effects: effects.clone(),
            remained_effects: remained_effects.clone(),
            accumulated_damage: accumulated_damage.to_vec(),
            damage_map: skill_list,
            extra: [0u8; 0],
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
            extra: [0u8; 0],
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
