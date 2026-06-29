use serde::{Deserialize, Serialize};

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
