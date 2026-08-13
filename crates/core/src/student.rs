use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomPinned};

use error::Error;
use typed_builder::TypedBuilder;

use serde::{Deserialize, Serialize};

use crate::{
    base::BaseStats,
    character::{Character, CharacterOps},
    constants::MAX_SKILL_LEVEL,
    locale::LocalizedName,
    skill::{FromParams, Skill},
    skills::kei::{KeiBasicSkill, KeiExSkill, KeiSubSkill},
    stat::{StatKind, StatValueKind},
    table::{
        gear::{GearKind, GearTable},
        level::calcul_stat,
    },
    terrains::{Terrain, TerrainCombatPower, TerrainCombatPowerState},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TypedBuilder)]
pub struct StudentSpec {
    pub id: u32,
    pub name: String,

    pub level: u8,
    pub star: u8,

    /// The elements in this array represent the levels of the following skills.
    /// Ex skill, Basic Skill, Enhanced Skill, Sub Skill
    pub skill_levels: [u8; 4],
    pub weapon_level: u8,
    pub bond_level: u8,

    /// Affinity Level of the Separated Character
    pub alter_bond_levels: Vec<u8>,

    pub gear_kinds: [GearKind; 3],
    pub gear_tiers: [u8; 3],
    pub gear_levels: [u8; 3],

    /// Each element in this array represents the following.
    /// Max HP Talent level, ATK Talent Level, Healing Talent Level
    pub talent_levels: [u8; 3],

    pub unique_item_level: Option<u8>,
}

/// Top level of `data/students/<student>.json`.
///
/// Growth from level, star tier and talent follows shared formulas and is not stored here. This
/// holds only the per-student values those formulas take as input.
#[derive(Debug, Clone, Deserialize)]
pub struct StudentFile {
    pub id: u32,
    pub name: LocalizedName,
    pub terrain_adaptation: TerrainCombatPower,

    /// The three gear kinds this student can equip. Their numbers live in the gear data.
    pub gear_slots: [GearKind; 3],

    /// Level 1 stats. `level` is a runtime value, absent from the file and left at 0.
    pub lvl1_stats: BaseStats,

    /// Level 1 and level 90 observations for each star tier.
    pub level_stats: StarCurves,
    pub unique_weapon: UniqueWeapon,

    /// Per-skill numbers. The fields differ by student, so this is left unopened here and read
    /// by that student's own crate into its `params` type.
    pub skills: serde_json::Value,
}

impl StudentFile {
    pub fn from_file(path: &str) -> Result<Self, error::Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct StarCurves {
    pub hp: StarValue<u64>,
    pub atk: StarValue<u32>,
    pub def: StarValue<u32>,
    pub healing: StarValue<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct StarValue<T> {
    pub lvl1: [T; 5],
    pub lvl90: [T; 5],
}

/// The four stats that grow with level. The rest do not, and `lvl1_stats` carries them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct LevelStats {
    pub hp: u64,
    pub atk: u32,
    pub def: u32,
    pub healing: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawStats {
    pub hp: f64,
    pub atk: f64,
    pub def: f64,
    pub healing: f64,
}

/// The student's unique weapon.
#[derive(Debug, Clone, Deserialize)]
pub struct UniqueWeapon {
    /// Values at levels 1, 30, 40, 50 and 60, which are the caps of each star tier.
    pub hp: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// Values at levels 1, 30, 40, 50 and 60, which are the caps of each star tier.
    pub atk: [u32; UniqueWeapon::MAX_STAR as usize + 1],

    /// Stat added at weapon star 2.
    pub star2_option: EnhancedSkillPlus,

    /// Terrain adaptation raised at weapon star 3, and the value it reaches.
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
    /// Maximum cost up by 0.5.
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

    /// Ex, Basic and Sub. The enhanced skill is always a stat increase, so it is folded into
    /// [`StudentStats::base_stats`] instead of being a skill.
    pub skills: Vec<Skill>,

    _pin: PhantomPinned,
}

impl Student {
    /// Returns a `Box` for the same reason [`crate::boss::Boss::from_file`] does: the skills hold
    /// a `NonNull` back to their owner, so the address has to be fixed before they are built.
    pub fn from_file(
        kind: StudentKind,
        path: &str,
        spec: StudentSpec,
        gears: &GearTable,
        skill_mask_offset: usize,
    ) -> Result<Box<Self>, Error> {
        let file = StudentFile::from_file(path)?;
        let skill_levels = spec.skill_levels;

        let base_stats = build_stats(&file, &spec, gears)?;

        let mut student = Box::new(Student {
            stats: StudentStats {
                student_stats: spec,
                base_stats,
            },
            skills: Vec::new(),
            _pin: PhantomPinned,
        });

        student.skills =
            build_skills(&student, kind, file.skills, skill_levels, skill_mask_offset)?;

        Ok(student)
    }
}

/// Level stats folded together with the gear, in the order the game applies them: every flat
/// increase is summed, then the summed rates multiply once.
fn build_stats(
    file: &StudentFile,
    spec: &StudentSpec,
    gears: &GearTable,
) -> Result<BaseStats, Error> {
    let raw = calcul_stat(
        spec.star,
        spec.level,
        spec.weapon_level,
        spec.talent_levels,
        file.unique_weapon.clone(),
        file.level_stats.clone(),
    )
    .ok_or_else(|| {
        Error::InvalidData(format!(
            "level {} is outside 1..=90 for {}",
            spec.level, spec.name
        ))
    })?;

    let mut mods: HashMap<StatKind, (f64, f64)> = HashMap::new();
    for i in 0..spec.gear_kinds.len() {
        let Some(stats) = gears.stats(
            spec.gear_kinds[i],
            spec.gear_tiers[i] as usize,
            spec.gear_levels[i] as usize,
        ) else {
            continue;
        };

        for stat in stats {
            let entry = mods.entry(stat.stat).or_insert((0.0, 0.0));
            match stat.kind {
                StatValueKind::Amount => entry.0 += stat.value.0,
                StatValueKind::Scale => entry.1 += stat.value.0,
            }
        }
    }

    let fold = |mods: &mut HashMap<StatKind, (f64, f64)>, stat, base: f64| {
        let (amount, scale) = mods.remove(&stat).unwrap_or((0.0, 0.0));
        ((base + amount) * (1.0 + scale / 100.0)).round()
    };

    let mut base_stats = file.lvl1_stats;
    base_stats.level = spec.level;
    base_stats.hp = fold(&mut mods, StatKind::Hp, raw.hp) as u64;
    base_stats.atk = fold(&mut mods, StatKind::Atk, raw.atk) as u32;
    base_stats.def = fold(&mut mods, StatKind::Def, raw.def) as u32;
    base_stats.healing = fold(&mut mods, StatKind::Healing, raw.healing) as u32;

    for (stat, (amount, scale)) in mods {
        base_stats = base_stats.apply_stat(stat, amount, 1.0 + scale / 100.0);
    }

    Ok(base_stats)
}

/// The one place [`StudentKind`] is tied to a Rust skill list.
fn build_skills(
    student: &Student,
    kind: StudentKind,
    skills: serde_json::Value,
    skill_levels: [u8; 4],
    offset: usize,
) -> Result<Vec<Skill>, Error> {
    let missing = |skill: &str, level: u8| {
        Error::InvalidData(format!("no data for {skill} skill at level {level}"))
    };

    match kind {
        StudentKind::Kei => {
            use crate::skills::kei::params::RawSkills;

            let raw: RawSkills = serde_json::from_value(skills)?;
            let [ex_lvl, basic_lvl, _, sub_lvl] = skill_levels;

            Ok(vec![
                Skill::KeiExSkill(KeiExSkill::new(
                    raw.ex.name.get(),
                    Character::Student(student),
                    offset,
                    raw.ex.pick(ex_lvl).ok_or_else(|| missing("ex", ex_lvl))?,
                )),
                Skill::KeiBasicSkill(KeiBasicSkill::new(
                    raw.basic.name.get(),
                    Character::Student(student),
                    offset + 1,
                    raw.basic
                        .pick(basic_lvl)
                        .ok_or_else(|| missing("basic", basic_lvl))?,
                )),
                Skill::KeiSubSkill(KeiSubSkill::new(
                    raw.sub.name.get(),
                    Character::Student(student),
                    offset + 2,
                    raw.sub
                        .pick(sub_lvl)
                        .ok_or_else(|| missing("sub", sub_lvl))?,
                )),
            ])
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudentKind {
    Kei,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillMeta;
    use crate::terrains::{Terrain, TerrainCombatPowerState};
    use crate::types::AttackType;

    const KEI: &str = "../../data/students/kei.json";
    const GEARS: &str = "../../data/tables/gears.json";

    fn spec(level: u8, star: u8) -> StudentSpec {
        StudentSpec::builder()
            .id(10135)
            .name("Kei".to_string())
            .level(level)
            .star(star)
            .skill_levels([5, 10, 10, 10])
            .weapon_level(0)
            .bond_level(1)
            .alter_bond_levels(Vec::new())
            .gear_kinds([GearKind::Shoes, GearKind::Hairpin, GearKind::Wristwatch])
            .gear_tiers([0, 0, 0])
            .gear_levels([0, 0, 0])
            .talent_levels([0, 0, 0])
            .unique_item_level(None)
            .build()
    }

    fn load(spec: StudentSpec) -> Box<Student> {
        let gears = GearTable::from_file(GEARS).expect("failed to load gears");
        Student::from_file(StudentKind::Kei, KEI, spec, &gears, 3).expect("failed to load kei")
    }

    /// With no gear, talent or weapon the endpoints must come back exactly as transcribed, or the
    /// interpolation is reading the wrong star tier.
    #[test]
    fn endpoints_reproduce_the_observations() {
        let file = StudentFile::from_file(KEI).expect("failed to load");

        for star in 1..=5u8 {
            let i = star as usize - 1;

            let at_1 = load(spec(1, star));
            assert_eq!(at_1.stats().hp, file.level_stats.hp.lvl1[i]);
            assert_eq!(at_1.stats().atk, file.level_stats.atk.lvl1[i]);
            assert_eq!(at_1.stats().def, file.level_stats.def.lvl1[i]);
            assert_eq!(at_1.stats().healing, file.level_stats.healing.lvl1[i]);

            let at_90 = load(spec(90, star));
            assert_eq!(at_90.stats().hp, file.level_stats.hp.lvl90[i]);
            assert_eq!(at_90.stats().atk, file.level_stats.atk.lvl90[i]);
            assert_eq!(at_90.stats().def, file.level_stats.def.lvl90[i]);
            assert_eq!(at_90.stats().healing, file.level_stats.healing.lvl90[i]);
        }
    }

    /// Talent is 0.2% of the **1-star** level 90 value per rank, not of the current tier.
    #[test]
    fn talent_scales_off_the_one_star_endpoint() {
        let file = StudentFile::from_file(KEI).expect("failed to load");

        let mut with_talent = spec(90, 5);
        with_talent.talent_levels = [10, 10, 10];

        let plain = load(spec(90, 5));
        let boosted = load(with_talent);

        let expected = |base: f64, one_star: f64| (base + one_star * 0.002 * 10.0).round();

        assert_eq!(
            boosted.stats().hp,
            expected(plain.stats().hp as f64, file.level_stats.hp.lvl90[0] as f64) as u64
        );
        assert_eq!(
            boosted.stats().atk,
            expected(
                plain.stats().atk as f64,
                file.level_stats.atk.lvl90[0] as f64
            ) as u32
        );
    }

    /// Gear stats must fold in with a single rounding at the end, and only the equipped slots
    /// may contribute.
    #[test]
    fn gear_folds_into_the_level_stats() {
        let mut equipped = spec(90, 5);
        equipped.gear_tiers = [7, 7, 7];
        equipped.gear_levels = [1, 1, 1];

        let plain = load(spec(90, 5));
        let with_gear = load(equipped);

        assert!(with_gear.stats().hp > plain.stats().hp);
        assert!(with_gear.stats().atk > plain.stats().atk);

        // Tier 0 is "not equipped" and must be skipped rather than treated as tier 1.
        let mut bare = spec(90, 5);
        bare.gear_levels = [10, 10, 10];
        assert_eq!(load(bare).stats().hp, plain.stats().hp);
    }

    /// Level 90, 5 stars, talent 25, no weapon and no gear. Reproduced exactly, which is what
    /// fixes the level and talent terms and leaves the weapon as the only suspect below.
    #[test]
    fn matches_the_external_calculator_without_weapon() {
        let mut talented = spec(90, 5);
        talented.talent_levels = [25, 25, 25];

        assert_eq!(load(talented).stats().hp, 19348);
    }

    /// The same student with the weapon at 60.
    ///
    /// `hp` lands one low. Given the test above pins level plus talent at 19348, the weapon's
    /// true value at level 60 has to sit somewhere in 2803.0 to 2803.5 while the table records
    /// the rounded 2803, and there is no way to recover that fraction from these observations.
    #[test]
    fn matches_the_external_calculator() {
        let mut maxed = spec(90, 5);
        maxed.weapon_level = 60;
        maxed.talent_levels = [25, 25, 25];

        let kei = load(maxed);

        assert_eq!(kei.stats().atk, 6345);
        assert!(
            kei.stats().hp.abs_diff(22152) <= 1,
            "hp = {}, expected 22152",
            kei.stats().hp
        );
    }

    /// The same student with three tier 10 gears at level 70: +35% hp, +46% atk, and 13000 flat
    /// hp from the hairpin.
    ///
    /// That `atk` is exact here confirms the single rounding at the end. Rounding the level
    /// stats first and multiplying afterwards would give 9264 as well, but only because the
    /// margin is under one; nothing in this data separates the two.
    #[test]
    fn matches_the_external_calculator_with_gear() {
        let mut maxed = spec(90, 5);
        maxed.weapon_level = 60;
        maxed.talent_levels = [25, 25, 25];
        maxed.gear_tiers = [10, 10, 10];
        maxed.gear_levels = [70, 70, 70];

        let kei = load(maxed);

        assert_eq!(kei.stats().atk, 9264);
        assert!(
            kei.stats().hp.abs_diff(47455) <= 1,
            "hp = {}, expected 47455",
            kei.stats().hp
        );
    }

    #[test]
    fn skills_are_built_at_the_given_offset() {
        let kei = load(spec(90, 5));

        assert_eq!(kei.skill_list().len(), 3);
        assert_eq!(kei.skill_list()[0].skill_mask_offset(), 3);
        assert_eq!(kei.skill_list()[2].skill_mask_offset(), 5);

        assert_eq!(kei.skill_list()[0].cost(), 2);
        assert_eq!(kei.skill_list()[0].duration(), 750);
        assert_eq!(kei.skill_list()[1].cost(), 0);
        assert_eq!(kei.skill_list()[1].frames(), 141);
    }

    /// The skills point back at the student they were built for. A move would break this.
    #[test]
    fn skills_point_at_their_owner() {
        let kei = load(spec(90, 5));

        for skill in kei.skill_list() {
            assert_eq!(skill.owner().id(), kei.id());
        }
    }

    #[test]
    fn load_kei() {
        let file = StudentFile::from_file("../../data/students/kei.json").expect("failed to load");

        assert_eq!(file.id, 10135);
        assert_eq!(file.lvl1_stats.attack_type, AttackType::Mystic);
        assert_eq!(file.lvl1_stats.level, 0);
        assert_eq!(file.gear_slots[0], GearKind::Shoes);
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
