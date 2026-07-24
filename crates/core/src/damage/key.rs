use std::{
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut},
};

/// 스킬이 적용되었는지 여부를 저장하는 구조체
///
/// MSB부터 보스여부 1비트, 자신인지 여부 1비트, 적인지 여부 1비트,
/// 각 학생의 스킬 적용 여부 `3 * 학생수` 비트, 보스 스킬 N비트입니다.
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

impl SkillsBitMask {
    pub const BOSS_BIT: u64 = 1u64 << 63;
    pub const SELF_BIT: u64 = 1u64 << 62;
    pub const ENEMY_BIT: u64 = 1u64 << 61;
    pub const DATA_MASK: u64 = (1u64 << Self::DATA_BITS_COUNT) - 1;

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
