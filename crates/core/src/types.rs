use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AttackType {
    #[serde(rename = "normal")]
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArmorType {
    #[serde(rename = "normal")]
    #[default]
    Normal,

    #[serde(rename = "light")]
    Light,

    #[serde(rename = "heavy")]
    Heavy,

    #[serde(rename = "composite")]
    Composite,

    #[serde(rename = "special")]
    Special,

    #[serde(rename = "elastic")]
    Elastic,

    #[serde(rename = "structure")]
    Structure,
}

/// The type matchup multiplier, per ten thousand.
///
/// A weak matchup is neutral here: its bonus is the attacker's effectiveness, added on top, which
/// makes 20000 at the default effectiveness of 10000.
pub fn damage_scale(atk_type: AttackType, armor_type: ArmorType) -> u32 {
    match atk_type {
        AttackType::Normal => 10000,
        AttackType::Explosive => match armor_type {
            ArmorType::Light | ArmorType::Normal | ArmorType::Heavy | ArmorType::Composite => 10000,
            ArmorType::Special | ArmorType::Elastic | ArmorType::Structure => 5000,
        },
        AttackType::Piercing => match armor_type {
            ArmorType::Light | ArmorType::Structure => 5000,
            _ => 10000,
        },
        AttackType::Corrosive => match armor_type {
            ArmorType::Heavy => 15000,
            ArmorType::Light | ArmorType::Structure => 5000,
            _ => 10000,
        },
        AttackType::Mystic => match armor_type {
            ArmorType::Heavy | ArmorType::Composite | ArmorType::Structure => 5000,
            _ => 10000,
        },
        AttackType::Sonic => match armor_type {
            ArmorType::Special => 15000,
            ArmorType::Elastic | ArmorType::Normal | ArmorType::Light => 10000,
            _ => 5000,
        },
    }
}

pub fn is_weak(atk_type: AttackType, armor_type: ArmorType) -> bool {
    (atk_type == AttackType::Explosive && armor_type == ArmorType::Light)
        || (atk_type == AttackType::Piercing && armor_type == ArmorType::Heavy)
        || (atk_type == AttackType::Mystic && armor_type == ArmorType::Special)
        || (atk_type == AttackType::Corrosive && armor_type == ArmorType::Composite)
        || (atk_type == AttackType::Sonic && armor_type == ArmorType::Elastic)
}
