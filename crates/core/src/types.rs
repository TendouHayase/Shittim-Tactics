use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    Normal,
    Explosive,
    Piercing,
    Corrosive,
    Mystic,
    Sonic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorType {
    Normal,
    Light,
    Heavy,
    Composite,
    Special,
    Elastic,
    Structure,
}
