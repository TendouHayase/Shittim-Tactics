use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Normal,
    Hard,
    VeryHard,
    Hardcore,
    Extreme,
    Insane,
    Torment,
    Lunatic,
}
