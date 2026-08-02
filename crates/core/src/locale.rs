//! 표시 언어. 실행 중 한 번 정해지면 끝까지 바뀌지 않음.

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

/// [`set_language`] 없이 먼저 읽을시 기본값(`Ko`)으로 반환됨.
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
