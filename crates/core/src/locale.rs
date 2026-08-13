//! Display language. Chosen once per run and never changed after.

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "ko")]
    #[default]
    Ko,

    #[serde(rename = "ja")]
    Ja,

    #[serde(rename = "en")]
    En,
}

static LANGUAGE: RwLock<Language> = RwLock::new(Language::Ko);

pub fn set_language(language: Language) -> Result<(), error::Error> {
    let mut guard = LANGUAGE.write()?;

    *guard = language;
    Ok(())
}

/// Read before [`set_language`], this yields the default of `Ko`.
pub fn language() -> Language {
    *LANGUAGE.read().expect("Failed to read language settings")
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalizedName {
    pub ko: String,
    pub ja: String,
    pub en: String,
}

impl LocalizedName {
    pub fn get(&self) -> &str {
        match language() {
            Language::Ko => &self.ko,
            Language::Ja => &self.ja,
            Language::En => &self.en,
        }
    }
}
