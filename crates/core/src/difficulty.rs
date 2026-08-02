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

    /// json의 난이도별 배열이 이 순서를 그대로 따름. [`ByDifficulty`]의 색인이 `as usize`라
    /// 순서를 바꾸면 기존 데이터가 전부 어긋나므로 변형을 중간에 끼워넣지 말 것. 길이가 맞지
    /// 않으면 컴파일이 깨지므로 [`Difficulty::COUNT`] 갱신을 잊을 수 없음.
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

/// 난이도순으로 늘어놓은 값. json의 스킬 수치는 전부 이 모양임.
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
