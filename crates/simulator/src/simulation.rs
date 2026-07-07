use core::{
    actions::{
        Action,
        ActionContext::{self},
    },
    boss::{Boss, BossTrait},
    simulator::Simulator,
    skill::{CasterContext, Skill, SkillContext, TargetContext},
    state::{
        State::{self, Assault, RestrictionRelease},
        StateData,
    },
    student::Student,
};
use std::rc::Rc;

pub struct Simulation<T: BossTrait> {
    pub students: Vec<Student>,
    pub boss: Boss<T>,
}

impl<T: BossTrait + Clone> Simulator<T> for Simulation<T> {
    fn legal_actions(&self, state: &core::state::State<T>) -> Vec<Rc<dyn Skill>> {
        match state {
            Assault(content) => {
                let cost = content.cost;
                let mut result = vec![];
                for (i, stat) in content.students.iter().enumerate() {
                    for (j, cooltime) in stat.stats.cooldowns.iter().enumerate() {
                        let skill = self.students[i].skills[j].clone();
                        if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                            result.push(skill);
                        }
                    }
                }

                result
            }
            RestrictionRelease(content) => {
                let cost = content.cost;
                let mut result = vec![];
                for (i, stat) in content.students.iter().enumerate() {
                    for (j, cooltime) in stat.stats.cooldowns.iter().enumerate() {
                        let skill = self.students[i].skills[j].clone();
                        if *cooltime == 0 && cost >= skill.cost().try_into().unwrap() {
                            result.push(skill);
                        }
                    }
                }

                result
            }
        }
    }

    fn apply(
        &self,
        state: &State<T>,
        action: &core::actions::ActionContext<dyn Skill>,
    ) -> core::state::State<T> {
        let action = match action {
            ActionContext::Wait => return state.clone(),
            ActionContext::Use(action) => action,
        };

        match state {
            Assault(data) => {
                let mut new_data = data.clone();
                Simulation::apply_action_to_data(&mut new_data, action);
                State::Assault(data.clone())
            }
            RestrictionRelease(data) => {
                let mut new_data = data.clone();
                Simulation::apply_action_to_data(&mut new_data, action);
                State::RestrictionRelease(data.clone())
            }
        }
    }
}

impl<T: BossTrait> Simulation<T> {
    pub fn build_skill_context<'a, const N: usize>(
        data: &'a mut StateData<N, T>,
        action: &'a Action<'a, dyn Skill>,
    ) -> SkillContext<'a> {
        let caster_id = action.caster.id();
        let caster_idx = Self::find_index::<N>(&data.students, caster_id);

        let target_ids: Vec<u32> = action.targets.iter().map(|t| t.id()).collect();

        let mut seen = std::collections::HashSet::new();
        seen.insert(caster_idx);
        for &id in &target_ids {
            if id != data.boss.stats.id {
                let idx = Self::find_index(&data.students, id);
                assert!(
                    seen.insert(idx),
                    "duplicate or self-targeting index detected; handle separately"
                );
            }
        }

        let ptr = data.students.as_mut_ptr();
        let caster_student = unsafe { &mut *ptr.add(caster_idx) };
        let caster_ctx = CasterContext::from_student(caster_student, action.skill.skill_type());

        let mut target_contexts = Vec::with_capacity(target_ids.len());
        for &id in &target_ids {
            if id == data.boss.stats.id {
                let boss_ref: &'a mut Boss<T> = unsafe { &mut *(&mut data.boss as *mut Boss<T>) };
                target_contexts.push(TargetContext::from_boss(boss_ref));
            } else {
                let idx = Self::find_index(&data.students, id);
                let student_ref: &'a mut Student = unsafe { &mut *ptr.add(idx) };
                target_contexts.push(TargetContext::from_student(student_ref));
            }
        }

        SkillContext {
            name: action.skill.name(),
            caster: caster_ctx,
            targets: target_contexts,
        }
    }

    fn apply_action_to_data<const N: usize>(
        data: &mut StateData<N, T>,
        action: &Action<dyn Skill>,
    ) {
        let mut skill_context = Self::build_skill_context(data, action);

        action
            .skill
            .apply(&mut skill_context.caster, &mut skill_context.targets);
    }

    fn find_index<const N: usize>(students: &[Student; N], id: u32) -> usize {
        students
            .iter()
            .position(|s| s.stats.student_stats.id == id)
            .expect("caster or target id not found in current state")
    }
}
