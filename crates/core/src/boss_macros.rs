// === xtask gen-skills: copied from bosses/src/macros.rs, do not edit by hand ===

/// create_boss_skill!(name: ident, cost: u8, duration: u16, frames: u16, skill_type: SkillType, skill_id: u8)
#[macro_export]
macro_rules! create_boss_skill {
    ($name:ident, $cost:literal, $duration:expr, $frames:expr, $skill_type:path, $skill_id:literal) => {
        #[derive(Debug)]
        pub struct $name {
            parent: NonNull<Boss>,
            skill_offset: usize,
            id: (u32, u8),
            name: String,
        }

        impl $name {
            pub fn name(&self) -> &str {
                &self.name
            }

            pub fn owner(&self) -> Character<'_> {
                unsafe { Character::Boss(self.parent.as_ref()) }
            }

            pub fn cost(&self) -> u8 {
                $cost
            }

            pub fn duration(&self) -> u16 {
                $duration
            }

            pub fn frames(&self) -> u16 {
                $frames
            }

            pub fn skill_mask_offset(&self) -> usize {
                self.skill_offset
            }

            pub fn skill_type(&self) -> SkillType {
                $skill_type
            }

            pub fn new(boss: &Boss, skill_mask_offset: usize) -> Self {
                Self {
                    parent: NonNull::from_ref(boss),
                    skill_offset: skill_mask_offset,
                    id: (boss.id(), $skill_id),
                    name: boss.stats.name.to_string(),
                }
            }
        }
    };
}
