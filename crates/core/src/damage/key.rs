use std::{
    collections::HashMap,
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

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

impl BitOr<u64> for DamageKey<'_> {
    type Output = Self;
    fn bitor(self, rhs: u64) -> Self::Output {
        DamageKey {
            mask: self.mask | rhs,
            skill_list: self.skill_list,
        }
    }
}

impl BitAnd<u64> for DamageKey<'_> {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        DamageKey {
            mask: self.mask & rhs,
            skill_list: self.skill_list,
        }
    }
}

impl BitOrAssign<u64> for DamageKey<'_> {
    fn bitor_assign(&mut self, rhs: u64) {
        self.mask = self.mask | rhs;
    }
}

impl BitAndAssign<u64> for DamageKey<'_> {
    fn bitand_assign(&mut self, rhs: u64) {
        self.mask = self.mask & rhs;
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

    #[inline]
    pub const fn is_boss(&self) -> bool {
        self.mask.is_boss()
    }

    #[inline]
    pub const fn is_self(&self) -> bool {
        self.mask.is_self()
    }

    #[inline]
    pub const fn is_enemy(&self) -> bool {
        self.mask.is_enemy()
    }

    #[inline]
    pub const fn data(&self) -> u64 {
        self.mask.data()
    }

    #[inline]
    pub fn clone_with_tag(&self, is_boss: bool, is_self: bool, is_enemy: bool) -> Self {
        let mut mask = self.mask.clone();

        // 조건이 true시 전항의 값은 0xFFFFFFFF, false시 0x00000000
        // 후항은 해당하는 비트 제외 모두 1
        // 조건 true시 등식 우측값은 0xFFFFFFFF, false시 해당하는 비트 제외 모두 1
        mask &= (0u64).wrapping_sub(is_boss.into()) | !SkillsBitMask::BOSS_BIT;
        mask &= (0u64).wrapping_sub(is_self.into()) | !SkillsBitMask::SELF_BIT;
        mask &= (0u64).wrapping_sub(is_enemy.into()) | !SkillsBitMask::ENEMY_BIT;

        Self {
            mask,
            skill_list: self.skill_list,
        }
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

impl BitOr<u64> for SkillsBitMask {
    type Output = Self;
    fn bitor(self, rhs: u64) -> Self::Output {
        SkillsBitMask(self.0 | (rhs & Self::DATA_MASK))
    }
}

impl BitAnd<u64> for SkillsBitMask {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        SkillsBitMask(self.0 & (rhs & Self::DATA_MASK))
    }
}

impl BitOrAssign<u64> for SkillsBitMask {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 = self.0 | (rhs & Self::DATA_MASK);
    }
}

impl BitAndAssign<u64> for SkillsBitMask {
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 = self.0 & (rhs & Self::DATA_MASK);
    }
}

impl SkillsBitMask {
    const BOSS_BIT: u64 = 1u64 << 63;
    const SELF_BIT: u64 = 1u64 << 62;
    const ENEMY_BIT: u64 = 1u64 << 61;
    const DATA_MASK: u64 = (1u64 << Self::DATA_BITS_COUNT) - 1;

    const DATA_BITS_COUNT: u64 = 61;

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
