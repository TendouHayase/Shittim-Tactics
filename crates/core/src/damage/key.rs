use std::{collections::HashMap, hash::Hash};

use crate::damage::Damage;

#[derive(Debug, Clone)]
pub struct DamageKey<'a> {
    pub mask: SkillsBitMask,
    skill_list: &'a HashMap<SkillsBitMask, Damage>,
}

impl PartialEq for DamageKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.mask == other.mask
    }
}

impl Eq for DamageKey<'_> {}

impl Hash for DamageKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.mask.hash(state);
    }
}

impl<'a> DamageKey<'a> {
    pub fn new(skill_list: &'a HashMap<SkillsBitMask, Damage>) -> Self {
        DamageKey {
            mask: Default::default(),
            skill_list,
        }
    }

    pub fn from_mask(mask: SkillsBitMask, other: &DamageKey<'a>) -> Self {
        Self {
            mask,
            skill_list: other.skill_list,
        }
    }

    pub fn damage(&self) -> Option<Damage> {
        self.skill_list.get(&self.mask).cloned()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct SkillsBitMask(pub u64);

impl From<u64> for SkillsBitMask {
    fn from(value: u64) -> Self {
        SkillsBitMask(value)
    }
}

impl SkillsBitMask {
    const BOSS_BIT: u64 = 1u64 << 63;
    const SELF_BIT: u64 = 1u64 << 62;
    const ENEMY_BIT: u64 = 1u64 << 61;
    const DATA_MASK: u64 = (1u64 << 61) - 1;

    pub const DATA_BITS_COUNT: u64 = 61;

    #[inline]
    pub const fn is_boss(&self) -> bool {
        (self.0 & SkillsBitMask::BOSS_BIT) != 0
    }

    #[inline]
    pub const fn is_self(&self) -> bool {
        (self.0 & SkillsBitMask::SELF_BIT) != 0
    }

    #[inline]
    pub const fn is_enemy(&self) -> bool {
        (self.0 & SkillsBitMask::ENEMY_BIT) != 0
    }

    #[inline]
    pub const fn data(&self) -> u64 {
        self.0 & SkillsBitMask::DATA_MASK
    }
}
