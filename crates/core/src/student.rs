use std::{fmt::Debug, hash::Hash, sync::Arc};

use error::Error;

use crate::{
    base::BaseStats,
    character::Character,
    extra::ExtraInit,
    skill::Skill,
    student::{
        file::StudentFile,
        spec::StudentSpec,
        stats::{StudentStats, build_stats},
    },
    table::gear::GearTable,
    uid::Uid,
};

pub mod file;
pub mod spec;
pub mod stats;

#[derive(Debug)]
pub struct Student {
    pub stats: StudentStats,

    /// Ex, Basic and Sub. The enhanced skill is always a stat increase, so it is folded into
    /// [`StudentStats::base_stats`] instead of being a skill.
    pub skills: Vec<Arc<dyn Skill>>,

    pub extra: Option<ExtraInit>,
}

impl Student {
    pub fn new(
        spec: StudentSpec,
        file: &StudentFile,
        gears: &GearTable,
        skills: Vec<Arc<dyn Skill>>,
        extra: Option<ExtraInit>,
    ) -> Result<Self, Error> {
        let base_stats = build_stats(file, &spec, gears)?;

        Ok(Student {
            stats: StudentStats {
                student_stats: spec,
                base_stats,
            },
            skills,
            extra,
        })
    }
}

impl Character for Student {
    fn uid(&self) -> Uid {
        self.stats.student_stats.uid
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skills(&self) -> &[Arc<dyn Skill>] {
        &self.skills
    }
}

impl Character for Box<Student> {
    fn uid(&self) -> Uid {
        self.stats.student_stats.uid
    }

    fn stats(&self) -> &BaseStats {
        &self.stats.base_stats
    }

    fn skills(&self) -> &[Arc<dyn Skill>] {
        &self.skills
    }
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.stats == other.stats
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
    use crate::table::gear::GearKind;
    use crate::terrains::{Terrain, TerrainCombatPowerState};
    use crate::types::AttackType;

    const KEI: &str = "../../data/students/kei.json";
    const GEARS: &str = "../../data/tables/gears.json";

    fn spec(level: u8, star: u8) -> StudentSpec {
        StudentSpec::builder()
            .uid(Uid::new(10135))
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
            .weapon_star(0)
            .build()
    }

    fn load(spec: StudentSpec) -> Student {
        let gears = GearTable::from_file(GEARS).expect("failed to load gears");
        let file = StudentFile::from_file(KEI).expect("failed to load kei");
        Student::new(spec, &file, &gears, Vec::new(), None).expect("failed to build kei")
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

    /// Weapon star 2 folds in the enhanced skill plus, which for this student is a flat hp
    /// amount.
    ///
    /// `curve` is indexed by enhanced skill level minus one, so level 1 takes the first entry
    /// and level 10 the last. Reading it without the offset would silently pick the next rank
    /// and go out of bounds at 10.
    #[test]
    fn weapon_star_two_adds_the_enhanced_skill_plus() {
        let kei = |weapon_star, enhanced_level| {
            let mut with_weapon = spec(90, 5);
            with_weapon.weapon_level = 40;
            with_weapon.weapon_star = weapon_star;
            with_weapon.skill_levels[2] = enhanced_level;
            load(with_weapon)
        };

        let star1 = kei(1, 10);
        let maxed = kei(2, 10);
        let lowest = kei(2, 1);

        assert_eq!(maxed.stats().hp - star1.stats().hp, 6020);
        assert_eq!(lowest.stats().hp - star1.stats().hp, 3168);

        // An amount on hp alone; the enhanced skill level moves nothing else.
        assert_eq!(maxed.stats().atk, star1.stats().atk);
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
