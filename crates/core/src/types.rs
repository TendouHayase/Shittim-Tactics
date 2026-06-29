use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    #[serde(rename = "normal")]
    Normal,

    #[serde(rename = "explosive")]
    Explosive,

    #[serde(rename = "piercing")]
    Piercing,

    #[serde(rename = "corrosive")]
    Corrosive,

    #[serde(rename = "mystic")]
    Mystic,

    #[serde(rename = "sonic")]
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
