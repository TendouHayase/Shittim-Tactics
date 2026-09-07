use std::{
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut},
};

/// Which skills are currently applied.
///
/// From the LSB: one bit each for boss, self and enemy, then `3 * students` bits for the
/// students' skills, then N bits for the boss's.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct SkillsBitMask(pub u64);

impl SkillsBitMask {
    pub const BOSS_BIT: u64 = 1u64;
    pub const SELF_BIT: u64 = 1u64 << 1;
    pub const ENEMY_BIT: u64 = 1u64 << 2;
    pub const DATA_MASK: u64 = !((1u64 << Self::DATA_BITS_COUNT) - 1);
    pub const TAG_MASK: u64 = (1 << Self::DATA_BITS_COUNT) - 1;

    const DATA_BITS_COUNT: u64 = 3;

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

    #[inline]
    pub const fn mask_enermy(self) -> Self {
        SkillsBitMask(self.0 & !SkillsBitMask::ENEMY_BIT)
    }

    #[inline]
    pub const fn mask_boss(self) -> Self {
        SkillsBitMask(self.0 & !SkillsBitMask::BOSS_BIT)
    }

    #[inline]
    pub const fn remove_flag(self, flag: SkillsBitMaskFlags) -> Self {
        SkillsBitMask(self.0 & !(flag as u64))
    }

    #[inline]
    pub fn clone_with_tag(&self, is_boss: bool, is_self: bool, is_enemy: bool) -> Self {
        let mut mask = *self;

        // 조건이 true시 전항의 값은 0xFFFFFFFF, false시 0x00000000
        // 후항은 해당하는 비트 제외 모두 1
        // 조건 true시 등식 우측값은 0xFFFFFFFF, false시 해당하는 비트 제외 모두 1
        mask &= (0u64).wrapping_sub(is_boss.into()) | !SkillsBitMask::BOSS_BIT;
        mask &= (0u64).wrapping_sub(is_self.into()) | !SkillsBitMask::SELF_BIT;
        mask &= (0u64).wrapping_sub(is_enemy.into()) | !SkillsBitMask::ENEMY_BIT;

        mask
    }
}

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
        SkillsBitMask(self.0 & (rhs | Self::TAG_MASK))
    }
}

impl BitOrAssign<u64> for SkillsBitMask {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 = self.0 | (rhs & Self::DATA_MASK);
    }
}

impl BitAndAssign<u64> for SkillsBitMask {
    fn bitand_assign(&mut self, rhs: u64) {
        self.0 = self.0 & (rhs | Self::TAG_MASK);
    }
}

impl Deref for SkillsBitMask {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SkillsBitMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for SkillsBitMask {
    type Item = SkillsBitMaskItem;
    type IntoIter = SkillsBitMaskIter;
    #[inline]
    fn into_iter(self) -> SkillsBitMaskIter {
        SkillsBitMaskIter(self.0)
    }
}

impl IntoIterator for &SkillsBitMask {
    type Item = SkillsBitMaskItem;
    type IntoIter = SkillsBitMaskIter;
    #[inline]
    fn into_iter(self) -> SkillsBitMaskIter {
        SkillsBitMaskIter(self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SkillsBitMaskItem(u64);

impl SkillsBitMaskItem {
    #[inline]
    pub const fn from_index(pos: u32) -> Self {
        debug_assert!(pos < u64::BITS);
        Self(1u64 << pos)
    }

    /// 내부 마스크를 그대로 반환.
    #[inline]
    pub const fn mask(self) -> u64 {
        self.0
    }

    /// 필요할 때만 위치를 계산.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0.trailing_zeros()
    }
}
struct SkillsBitMaskIter(u64);

impl Iterator for SkillsBitMaskIter {
    type Item = SkillsBitMaskItem;

    #[inline]
    fn next(&mut self) -> Option<SkillsBitMaskItem> {
        if self.0 == 0 {
            return None;
        }
        let lowest = self.0 & self.0.wrapping_neg();
        self.0 &= self.0 - 1;
        Some(SkillsBitMaskItem(lowest))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for SkillsBitMaskIter {
    #[inline]
    fn next_back(&mut self) -> Option<SkillsBitMaskItem> {
        if self.0 == 0 {
            return None;
        }
        let highest = 1u64 << (u64::BITS - 1 - self.0.leading_zeros());
        self.0 ^= highest;
        Some(SkillsBitMaskItem(highest))
    }
}

impl ExactSizeIterator for SkillsBitMaskIter {}

#[repr(u64)]
pub enum SkillsBitMaskFlags {
    BossBit = SkillsBitMask::BOSS_BIT,
    SelfBit = SkillsBitMask::SELF_BIT,
    EnemyBit = SkillsBitMask::ENEMY_BIT,
}
