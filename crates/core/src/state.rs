use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use crate::{
    character::{Character, CharacterOps},
    damage::{Damage, cache::DamageCache, key::SkillsBitMask},
    states::MAX_EXTRA_STATE_SIZE,
    utils::Position,
};

pub trait Stateful<'a>: Clone + Send + Sync + Eq + Hash {
    fn new(students: &[StateData<'a>], boss: StateData<'a>, elased_frames: u16, cost: i8) -> Self;
    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a>];
    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a>];
    fn boss<'b: 'c, 'c>(&'b self) -> &'c StateData<'a>;
    fn boss_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut StateData<'a>;
    /// 보스와 학생들을 단 한 번의 가변 대여로 동시에 꺼냅니다.
    fn split_mut<'b>(&'b mut self) -> (&'b mut StateData<'a>, &'b mut [StateData<'a>]);
    fn cost(&self) -> i8;
    fn frames(&self) -> u16;
    fn is_terminated(&self) -> bool;
    fn is_goal(&self, threshold_percent: f64) -> bool;
    fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32) -> Option<&'c StateData<'a>>;
    fn state_data_by_id_mut<'b: 'c, 'c>(&'b mut self, id: u32) -> Option<&'c mut StateData<'a>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State<'a> {
    pub students: StudentState<'a>,
    pub boss: StateData<'a>,
    pub frames: u16,
    pub cost: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StudentState<'a> {
    TotalAssault([StateData<'a>; 6]),
    FinalRestrictionRelease([StateData<'a>; 10]),
}

impl<'a> Stateful<'a> for State<'a> {
    fn new(students: &[StateData<'a>], boss: StateData<'a>, frames: u16, cost: i8) -> Self {
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

    fn students<'b: 'c, 'c>(&'b self) -> &'c [StateData<'a>] {
        match &self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
    }

    fn students_mut<'b: 'c, 'c>(&'b mut self) -> &'c mut [StateData<'a>]
    where
        'a: 'b,
        'b: 'c,
    {
        match &mut self.students {
            StudentState::TotalAssault(arr) => arr,
            StudentState::FinalRestrictionRelease(arr) => arr,
        }
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

    fn split_mut<'b>(&'b mut self) -> (&'b mut StateData<'a>, &'b mut [StateData<'a>]) {
        // `self.students_mut()`를 쓰면 self 전체를 다시 대여하게 되므로
        // 반드시 필드를 직접 매치해야 서로 겹치지 않는 대여로 인정받습니다.
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

    fn state_data_by_id<'b: 'c, 'c>(&'b self, id: u32) -> Option<&'c StateData<'a>> {
        if id == self.boss.character.id() {
            return Some(&self.boss);
        }

        self.students()
            .iter()
            .find(|&student| id == student.character.id())
            .map(|v| v as _)
    }

    fn state_data_by_id_mut<'b: 'c, 'c>(&'b mut self, id: u32) -> Option<&'c mut StateData<'a>> {
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StateData<'a> {
    pub cooldowns: Vec<u16>,
    pub remained_effects: BinaryHeap<Reverse<RemainedEffects>>,
    pub accumulated_damage: Vec<AccumulatedDamage>,

    pub damage_map: &'a HashMap<SkillsBitMask, Damage>,
    pub character: &'a Character<'a>,
    pub effects: SkillsBitMask,
    pub accumulated_damage_cache: DamageCache,

    pub coordinate: Position,

    pub extra: [u8; MAX_EXTRA_STATE_SIZE],
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
            (self.character as *const Character) as *const usize as usize,
            &self.cooldowns,
            &self.effects,
            &self.accumulated_damage,
            self.coordinate,
        )
            .hash(state);
    }
}

impl<'a> StateData<'a> {
    #[allow(unsafe_code)]
    pub fn extra_as<T>(&self) -> &T {
        const {
            assert!(::std::mem::size_of::<T>() <= MAX_EXTRA_STATE_SIZE);
            assert!(::std::mem::offset_of!(Self, extra) % ::std::mem::align_of::<T>() == 0);
        };
        unsafe { &*(self.extra.as_ptr() as *const T) }
    }

    #[allow(unsafe_code)]
    pub fn extra_as_mut<T>(&mut self) -> &mut T {
        const {
            assert!(::std::mem::size_of::<T>() <= MAX_EXTRA_STATE_SIZE);
            assert!(::std::mem::offset_of!(Self, extra) % ::std::mem::align_of::<T>() == 0);
        };
        unsafe { &mut *(self.extra.as_mut_ptr() as *mut T) }
    }
}

impl<'a> StateData<'a> {
    pub fn new(
        character: &'a Character<'_>,
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
            extra: [0u8; MAX_EXTRA_STATE_SIZE],
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
        extra: [u8; MAX_EXTRA_STATE_SIZE],
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
            extra,
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
            extra: self.extra,
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
