pub mod kei;
pub mod skill;

use core::{
    extra::ExtraInit,
    simulator::Simulator,
    skill::Skill,
    student::{Student, file::StudentFile, spec::StudentSpec},
    table::gear::GearTable,
    uid::Uid,
};
use error::Error;
use kei::{
    skill::{KeiBasicSkill, KeiExSkill, KeiSubSkill, params::RawSkills},
    state::KeiState,
};
use serde::Deserialize;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudentKind {
    Kei,
}

pub fn load(
    kind: StudentKind,
    path: &str,
    spec: StudentSpec,
    gears: &GearTable,
    skill_offset: usize,
    sim: Weak<Simulator>,
) -> Result<Student, Error> {
    let file = StudentFile::from_file(path)?;
    let skills = build_skills(
        kind,
        spec.uid,
        &file.skills,
        spec.skill_levels,
        skill_offset,
        sim,
    )?;

    let extra: Option<ExtraInit> = match kind {
        StudentKind::Kei => Some(core::extra::init::<KeiState>),
    };

    Student::new(spec, &file, gears, skills, extra)
}

fn build_skills(
    kind: StudentKind,
    owner: Uid,
    skills: &serde_json::Value,
    skill_levels: [u8; 4],
    offset: usize,
    sim: Weak<Simulator>,
) -> Result<Vec<Arc<dyn Skill>>, Error> {
    let missing = |skill: &str, level: u8| {
        Error::InvalidData(format!("no data for {skill} skill at level {level}"))
    };

    Ok(match kind {
        StudentKind::Kei => {
            let raw = RawSkills::deserialize(skills)?;
            let [ex_lvl, basic_lvl, _, sub_lvl] = skill_levels;

            vec![
                Arc::new(KeiExSkill::new(
                    owner,
                    raw.ex.name.get(),
                    offset,
                    raw.ex.pick(ex_lvl).ok_or_else(|| missing("ex", ex_lvl))?,
                    sim.clone(),
                )),
                Arc::new(KeiBasicSkill::new(
                    owner,
                    raw.basic.name.get(),
                    offset + 1,
                    raw.basic
                        .pick(basic_lvl)
                        .ok_or_else(|| missing("basic", basic_lvl))?,
                    sim.clone(),
                )),
                Arc::new(KeiSubSkill::new(
                    owner,
                    raw.sub.name.get(),
                    offset + 2,
                    raw.sub
                        .pick(sub_lvl)
                        .ok_or_else(|| missing("sub", sub_lvl))?,
                    sim.clone(),
                )),
            ]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{character::Character, table::gear::GearKind};

    const KEI: &str = "../../data/students/kei.json";
    const GEARS: &str = "../../data/tables/gears.json";

    fn load_kei() -> Student {
        let spec = StudentSpec::builder()
            .uid(Uid::new(10135))
            .name("Kei".to_string())
            .level(90)
            .star(5)
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
            .build();
        let gears = GearTable::from_file(GEARS).expect("failed to load gears");

        load(StudentKind::Kei, KEI, spec, &gears, 3, Weak::default()).expect("failed to load kei")
    }

    #[test]
    fn skills_are_built_at_the_given_offset() {
        let kei = load_kei();

        assert_eq!(kei.skills().len(), 3);
        assert_eq!(kei.skills()[0].skill_offset(), 3);
        assert_eq!(kei.skills()[2].skill_offset(), 5);

        assert_eq!(kei.skills()[0].cost(), 2);
        assert_eq!(kei.skills()[0].duration(), 750);
        assert_eq!(kei.skills()[1].cost(), 0);
        assert_eq!(kei.skills()[1].frames(), 141);
    }

    #[test]
    fn skills_point_at_their_owner() {
        let kei = load_kei();

        for skill in kei.skills() {
            assert_eq!(skill.owner_uid(), kei.uid());
        }
    }
}
