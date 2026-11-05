pub const TPS: u16 = 30;
/// Fixed-point units in one cost.
///
/// Recovery is given per second in ten-thousandths of a cost. Scaling by `TPS` as well makes one
/// tick worth exactly the per-second recovery value, so regeneration never rounds.
pub const COST_UNIT: u32 = 10_000 * TPS as u32;
pub const MAX_COST: u32 = 10 * COST_UNIT;
pub const TOTAL_ASSAULT_HAND_SIZE: usize = 3;
pub const FINAL_RESTRICTION_RELEASE_HAND_SIZE: usize = 5;
pub const MAX_STUDENT_COUNT: usize = 10;
pub const MAX_SKILL_LEVEL: usize = 10;
pub const MAX_EX_SKILL_LEVEL: usize = 5;
