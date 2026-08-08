use std::{fmt::Debug, hash::Hash, marker::PhantomPinned};

use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{
    base::BaseStats,
    character::CharacterOps,
    constants::MAX_SKILL_LEVEL,
    locale::LocalizedName,
    skill::Skill,
    stat::{StatKind, StatValueKind},
    table::gear::GearKind,
    terrains::{Terrain, TerrainCombatPower, TerrainCombatPowerState},
    utils::Ratio,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,

    /// The elements in this array represent the levels of the following skills.
    /// Ex skill, Basic Skill, Enhanced Skill, Sub Skill
    pub skill_levels: [u8; 4],
    pub weapon_level: u8,
    pub bond_level: u8,

    /// Affinity Level of the Separated Character
    pub alter_bond_levels: Vec<u8>,

    pub gear_tiers: [u8; 3],

    pub gear_levels: [u8; 3],

    /// Each element in this array represents the following.
    /// Max HP Talent level, ATK Talent Level, Healing Talent Level
    pub talent_levels: [u8; 3],

    pub unique_item_level: Option<u8>,
}

/// `data/students/<학생>.json`의 최상위.
///
/// 레벨·성급·성작·능력개방에 따른 증가는 전부 공용 수식이라 여기 없음. 이 파일에 있는 것은
/// 그 수식의 입력이 되는 학생 고유값뿐임.
#[derive(Debug, Clone, Deserialize)]
pub struct StudentFile {
    pub id: u32,
    pub name: LocalizedName,
    pub terrain_adaptation: TerrainCombatPower,

    /// 이 학생이 낄 수 있는 장비 3종. 수치는 여기 없고 장비 쪽 데이터에 있음.
    pub gear_slots: [GearKind; 3],

    /// 1레벨 스탯. `level`은 런타임 값이라 파일에 없고 0으로 들어옴.
    pub lvl1_stats: BaseStats,

    pub delta: GrowthDelta,
    pub stats_at_90: LevelStats,
    pub unique_weapon: UniqueWeapon,

    /// 스킬별 수치. 학생마다 필드가 달라 여기서는 열어보지 않고, 해당 학생 크레이트가
    /// 자기 `params` 구조체로 읽음.
    pub skills: serde_json::Value,
}

/// 레벨 1당 증가량. `lvl1_stats`와 `stats_at_90` 두 끝점에서 유도되는 값이지만 게임이
/// 표시하는 자릿수 그대로를 담고 있어 그 나눗셈과 정확히 일치하지는 않음.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct GrowthDelta {
    pub hp: Ratio,
    pub atk: Ratio,
    pub def: Ratio,
    pub healing: Ratio,
}

/// 레벨에 따라 자라는 네 스탯만. 나머지는 레벨과 무관해 `lvl1_stats`가 그대로 쓰임.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct LevelStats {
    pub hp: u64,
    pub atk: u32,
    pub def: u32,
    pub healing: u32,
}

/// 고유무기/전용무기
#[derive(Debug, Clone, Deserialize)]
pub struct UniqueWeapon {
    /// [1렙시 수치, 1성 최대 레벨시 수치(30레벨), 2성 최대 레벨시 수치(40레벨)..]
    pub hp: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// [1렙시 수치, 1성 최대 레벨시 수치(30레벨), 2성 최대 레벨시 수치(40레벨)..]
    pub atk: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// 고유무기 2성에 추가되는 스탯
    pub star2_option: EnhancedSkillPlus,

    /// 고유무기 3성에 증가되는 지형 적성과 그 값
    pub star3_option: (Terrain, TerrainCombatPowerState),

    pub star4_option: UniqueWeapon4StarOption,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnhancedSkillPlus {
    pub stat: StatKind,
    pub kind: StatValueKind,
    pub curve: [u32; MAX_SKILL_LEVEL],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UniqueWeapon4StarOption {
    /// 최대 코스트 0.5 증가
    MaxCostUp,
    ExplosiveEffectiveness,
    PiercingEffectiveness,
    CorrosiveEffectiveness,
    MysticEffectiveness,
    SonicEffectiveness,
}

impl UniqueWeapon {
    pub const MAX_STAR: u8 = 4;
}

#[derive(Debug, Clone)]
pub struct StudentStats {
    pub student_stats: StudentSpec,
    pub base_stats: BaseStats,
}

impl PartialEq for StudentStats {
    fn eq(&self, other: &Self) -> bool {
        self.student_stats.id == other.student_stats.id
    }
}

impl Eq for StudentStats {}

impl Hash for StudentStats {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.student_stats.id.hash(state);
    }
}

#[derive(Debug)]
pub struct Student {
    pub stats: StudentStats,

    /// Ex, Basic, Sub. 강화스킬은 늘 수치 증가라 스킬로 두지 않고 [`StudentStats::base_stats`]에
    /// 미리 접어넣음.
    pub skills: [Skill; 3],

    _pin: PhantomPinned,
}

impl StudentFile {
    pub fn from_file(path: &str) -> Result<Self, error::Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }
}

impl Student {
    pub fn from_file() -> Result<Box<Self>, error::Error> {
        todo!()
    }
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
    }
}

impl CharacterOps for Student {
    fn id(&self) -> u32 {
        self.stats.student_stats.id
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skill_list(&self) -> &[Skill] {
        &self.skills
    }
}

impl Hash for Student {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stats.hash(state);
    }
}

impl Eq for Student {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrains::{Terrain, TerrainCombatPowerState};
    use crate::types::AttackType;

    #[test]
    fn load_kei() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");

        assert_eq!(file.id, 10135);
        assert_eq!(file.lvl1_stats.attack_type, AttackType::Mystic);
        assert_eq!(file.lvl1_stats.level, 0);
        assert_eq!(file.stats_at_90.hp, 15479);
        assert_eq!(file.gear_slots[0], GearKind::Shoes);
    }

    /// 소수가 `f64`를 거쳐도 자릿수 그대로 복원되는지. 여기가 깨지면 스탯이 조용히 몇씩
    /// 어긋남.
    #[test]
    fn delta_keeps_decimals() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");

        assert_eq!(file.delta.def.num(), 35);
        assert_eq!(file.delta.def.den(), 10);
        assert_eq!(file.delta.healing.num(), 268);
        assert_eq!(file.delta.hp.den(), 1);

        // 89레벨분을 한 번에 곱하므로 레벨마다 버리는 것과 결과가 다름.
        assert_eq!(file.delta.def.apply(89), 311);
    }

    #[test]
    fn unique_weapon_promotes_the_top_terrain() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");
        let mut adaptation = file.terrain_adaptation;

        assert_eq!(adaptation.get(Terrain::Street), TerrainCombatPowerState::S);
        adaptation.promote_best();

        assert_eq!(adaptation.get(Terrain::Street), TerrainCombatPowerState::SS);
        assert_eq!(adaptation.get(Terrain::Indoor), TerrainCombatPowerState::A);
    }
}
