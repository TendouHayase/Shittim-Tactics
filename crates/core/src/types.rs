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

pub fn damage_scale(atk_type: &AttackType, armor_type: &ArmorType) -> u32 {
    match atk_type {
        AttackType::Normal => 100,
        AttackType::Explosive => match armor_type {
            ArmorType::Light => 200,
            ArmorType::Normal | ArmorType::Heavy | ArmorType::Composite => 100,
            ArmorType::Special | ArmorType::Elastic | ArmorType::Structure => 50,
        },
        AttackType::Piercing => match armor_type {
            ArmorType::Heavy => 200,
            ArmorType::Light | ArmorType::Structure => 50,
            _ => 100,
        },
        AttackType::Corrosive => match armor_type {
            ArmorType::Composite => 200,
            ArmorType::Heavy => 150,
            ArmorType::Light | ArmorType::Structure => 50,
            _ => 100,
        },
        AttackType::Mystic => match armor_type {
            ArmorType::Special => 200,
            ArmorType::Heavy | ArmorType::Composite | ArmorType::Structure => 50,
            _ => 100,
        },
        AttackType::Sonic => match armor_type {
            ArmorType::Elastic => 200,
            ArmorType::Special => 150,
            ArmorType::Normal | ArmorType::Light => 100,
            _ => 50,
        },
    }
}

pub fn is_weak(atk_type: AttackType, armor_type: ArmorType) -> bool {
    if atk_type == AttackType::Explosive && armor_type == ArmorType::Light {
        true
    } else if atk_type == AttackType::Piercing && armor_type == ArmorType::Heavy {
        true
    } else if atk_type == AttackType::Mystic && armor_type == ArmorType::Special {
        true
    } else if atk_type == AttackType::Corrosive && armor_type == ArmorType::Composite {
        true
    } else {
        atk_type == AttackType::Sonic && armor_type == ArmorType::Elastic
    }
}
