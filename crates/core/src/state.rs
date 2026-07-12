use std::collections::{BinaryHeap, HashMap, LinkedList, VecDeque};

use crate::{
    Position,
    character::Character,
    damage::{
        Damage,
        cache::DamageCache,
        key::{DamageKey, SkillsBitMask},
    },
};

#[repr(align(64))]
#[derive(Debug, Clone, PartialEq)]
pub struct State<'a, const N: usize> {
    pub students: [StateData<'a>; N],
    pub cost: i8,
    pub frames: u16,
    pub boss: StateData<'a>,
}

#[repr(align(64))]
#[derive(Debug, Clone)]
pub struct StateData<'a> {
    pub character: &'a dyn Character,

    /// These are the student's coordinates.
    pub coordinate: Position,
    pub accumulated_damage_cache: DamageCache,
    pub cooldowns: Vec<u16>,
    pub effects: DamageKey<'a>,

    /// (남은 지속틱, 해당 비트)
    pub remained_effects: BinaryHeap<RemainedEffects>,

    /// (데미지, 지속 틱)
    pub accumulated_damage: Vec<AccumulatedDamage<'a>>,
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

pub trait Stateful<'a>: Clone {
    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a>];
    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a>];
    fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a>;
    fn boss_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut StateData<'a>;
    fn cost(&self) -> i8;
    fn frames(&self) -> u16;
}

impl PartialOrd for RemainedEffects {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RemainedEffects {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.ticks.cmp(&self.ticks)
    }
}

impl<'a, const N: usize> Stateful<'a> for State<'a, N> {
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
}

impl PartialEq for StateData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cooldowns == other.cooldowns
            && self.effects == other.effects
            && self.accumulated_damage == other.accumulated_damage
            && self.accumulated_damage_cache == other.accumulated_damage_cache
    }
}

impl<'a> StateData<'a> {
    pub unsafe fn new(
        character: &'a dyn Character,
        skill_list: &'a HashMap<SkillsBitMask, Damage>,
    ) -> Self {
        StateData {
            character: character,
            coordinate: Default::default(),
            cooldowns: vec![0; character.skill_list().len()],
            effects: DamageKey::new(skill_list),
            remained_effects: BinaryHeap::new(),
            accumulated_damage: Vec::new(),
            accumulated_damage_cache: Default::default(),
        }
    }

    pub fn from_parts<'b>(
        character: &'a dyn Character,
        coordinate: Position,
        cooldowns: &[u16],
        effects: &'b DamageKey<'a>,
        remained_effects: &'b BinaryHeap<RemainedEffects>,
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
            accumulated_damage: accumulated_damage.iter().cloned().collect(),
        }
    }

    pub fn clone_matching<'b>(
        &self,
        cooldowns_condition: impl Fn(&u16) -> u16,
        effects: DamageKey<'a>,
        remained_effects: BinaryHeap<RemainedEffects>,
    ) -> Self
    where
        'a: 'b,
    {
        StateData {
            character: self.character,
            coordinate: self.coordinate,
            accumulated_damage_cache: self.accumulated_damage_cache.clone(),
            cooldowns: self
                .cooldowns
                .iter()
                .map(|i| cooldowns_condition(i))
                .collect(),
            effects: effects,
            remained_effects: remained_effects,
            accumulated_damage: self.accumulated_damage.clone(),
        }
    }
}
