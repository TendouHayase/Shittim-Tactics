use std::collections::HashMap;

use crate::damage::{Damage, key::SkillsBitMask};

#[derive(Debug)]
pub struct DamageMap {
    entries: HashMap<SkillsBitMask, Damage>,
}

impl DamageMap {
    pub fn get(&self, mask: SkillsBitMask) -> Option<Damage> {
        self.entries.get(&mask).copied()
    }

    pub fn max_damage(&self) -> Option<Damage> {
        self.entries.values().max().copied() // 데미지 감소 적용 필요
    }
}

impl FromIterator<(SkillsBitMask, Damage)> for DamageMap {
    fn from_iter<T: IntoIterator<Item = (SkillsBitMask, Damage)>>(iter: T) -> Self {
        DamageMap {
            entries: iter.into_iter().collect(),
        }
    }
}
