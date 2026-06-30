use core::{
    actions::Action::{self, Use},
    boss::BossSpec,
    skill::Skill,
    state::State,
    student::Student,
};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommonState<'a, S: State> {
    pub students: &'a Vec<Student>,
    pub cost: i8,
    pub boss: Rc<BossSpec>,
    pub state: S,
}

impl<'a, S: State> State for CommonState<'a, S> {
    fn is_terminal(&self) -> bool {
        for stud in self.students {
            if stud.stats.base_stats.hp != 0 {
                return false;
            }
        }

        return true;
    }

    fn is_goal(&self) -> bool {
        self.boss.base_stats.hp == 0
    }

    fn g_cost(&self) -> f32 {
        todo!()
    }

    fn mut_status(&mut self) -> &mut Self {
        self
    }

    fn status(&self) -> &Self {
        self
    }
}
