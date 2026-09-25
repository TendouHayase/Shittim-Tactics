use std::collections::HashMap;

use error::Error;
use serde::Deserialize;

use crate::{difficulty::Difficulty, locale::LocalizedName, types::ArmorType};

/// Top level of `data/bosses/<boss>.json`.
///
/// Armor type keys differ per boss, so every remaining key is swept up. Any top-level key that
/// is not an armor type, such as `skills`, must therefore be declared as a field here; leaving
/// one out surfaces as an `ArmorType` parse failure.
#[derive(Debug, Deserialize)]
pub struct BossFile {
    pub id: u32,
    pub name: LocalizedName,
    pub skills: serde_json::Value,

    #[serde(flatten)]
    pub(super) by_armor: HashMap<ArmorType, HashMap<Difficulty, super::DifficultyEntry>>,
}

impl BossFile {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        parsing_json::read_json(path)
    }
}
