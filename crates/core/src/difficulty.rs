use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    #[serde(rename = "normal")]
    Normal,

    #[serde(rename = "hard")]
    Hard,

    #[serde(rename = "veryhard")]
    VeryHard,

    #[serde(rename = "hardcore")]
    Hardcore,

    #[serde(rename = "extreme")]
    Extreme,

    #[serde(rename = "insane")]
    Insane,

    #[serde(rename = "torment")]
    Torment,

    #[serde(rename = "lunatic")]
    Lunatic,
}

impl Difficulty {
    pub const COUNT: usize = 8;

    /// Per-difficulty arrays in json follow this order, and [`ByDifficulty`] indexes with
    /// `as usize`, so inserting a variant in the middle silently shifts every existing value.
    /// A wrong length breaks the build, so [`Difficulty::COUNT`] cannot be forgotten.
    pub const ALL: [Difficulty; Self::COUNT] = [
        Difficulty::Normal,
        Difficulty::Hard,
        Difficulty::VeryHard,
        Difficulty::Hardcore,
        Difficulty::Extreme,
        Difficulty::Insane,
        Difficulty::Torment,
        Difficulty::Lunatic,
    ];
}

/// Values laid out in difficulty order. Every skill number in json has this shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ByDifficulty<T>([T; Difficulty::COUNT]);

impl<T> Index<Difficulty> for ByDifficulty<T> {
    type Output = T;

    fn index(&self, difficulty: Difficulty) -> &Self::Output {
        &self.0[difficulty as usize]
    }
}

impl<T> From<[T; Difficulty::COUNT]> for ByDifficulty<T> {
    fn from(value: [T; Difficulty::COUNT]) -> Self {
        Self(value)
    }
}
