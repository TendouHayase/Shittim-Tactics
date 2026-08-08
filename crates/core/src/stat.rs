use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatKind {
    Hp,
    Atk,
    Def,
    Healing,
    Accuracy,
    Evasion,
    Crit,
    CritRes,
    CritDmg,
    CritDmgRes,
    Stability,
    StabilityRate,
    NormalAttackRange,
    SightingRange,
    CcPower,
    CcRes,
    RecoveryBoost,
    CostRecovery,
    AtkSpeed,
    MovSpeed,
    BlockRateBonus,
    DefensePiercing,
    MagCount,
    DmgDealt,
    DmgResist,
    ExSkillDmgDealt,
    ExSkillDmgResist,
    BasicsProficiency,
    HealingBoost,
    ExplosiveEffectiveness,
    PiercingEffectiveness,
    CorrosiveEffectiveness,
    MysticEffectiveness,
    SonicEffectiveness,
    BuffRetention,
    DebuffRetention,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatValueKind {
    Amount,
    Scale,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Stat {
    pub stat: StatKind,
    pub kind: StatValueKind,
    pub value: OrderedFloat<f64>,
}
