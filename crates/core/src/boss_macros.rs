// === xtask gen-skills: copied from bosses/src/macros.rs, do not edit by hand ===

/// create_boss_skill!(name: ident, cost: u8, duration: u16, frames: u16, skill_type: SkillType, skill_id: u8, { 나머지 SkillOps 메서드 })
///
/// 마지막 블록에는 이 매크로가 만들어주지 않는 `SkillOps` 메서드(`skill_effects`, `apply`)를
/// 넣는다. 트레이트 impl은 여러 블록으로 쪼갤 수 없어서 호출부가 따로 `impl SkillOps for`를
/// 열 수 없기 때문에, 나머지 메서드도 이 매크로가 생성하는 impl 안으로 들어와야 한다.
/// `new`처럼 `SkillOps`에 없는 항목은 별도 inherent impl로 나간다.
#[macro_export]
macro_rules! create_boss_skill {
    (
        $name:ident,
        $cost:literal,
        $duration:expr,
        $frames:expr,
        $skill_type:path,
        $skill_id:literal,
        { $($rest:tt)* }
    ) => {


        #[derive(Debug)]
        pub struct $name {
            parent: NonNull<Boss>,
            skill_offset: usize,
            id: (u32, u8),
            name: String,
        }

        impl $name {
            pub fn new(boss: &Boss, skill_mask_offset: usize) -> Self {
                Self {
                    parent: NonNull::from_ref(boss),
                    skill_offset: skill_mask_offset,
                    id: (boss.id(), $skill_id),
                    name: boss.stats.name.to_string(),
                }
            }
        }

        impl SkillOps for $name {
            fn name(&self) -> &str {
                &self.name
            }

            fn owner(&self) -> Character<'_> {
                unsafe { Character::Boss(self.parent.as_ref()) }
            }

            fn cost(&self) -> u8 {
                $cost
            }

            fn duration(&self) -> u16 {
                $duration
            }

            fn frames(&self) -> u16 {
                $frames
            }

            fn skill_mask_offset(&self) -> usize {
                self.skill_offset
            }

            fn skill_type(&self) -> SkillType {
                $skill_type
            }

            $($rest)*
        }
    };
}
