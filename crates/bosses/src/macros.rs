/// Declares a boss skill struct together with its `SkillMeta` implementation.
///
/// ```ignore
/// create_boss_skill!(Name, params: <Params>, SkillType::Ex, 0, { /* SkillOps methods */ });
/// create_boss_skill!(Name, cost, duration, frames, SkillType::Ex, 0, { /* ... */ });
/// create_boss_skill!(Name, cost, duration, frames, SkillType::Ex, 0, params: <Params>, { /* ... */ });
/// ```
///
/// The trailing block holds the `SkillOps` methods (`skill_effects`, `apply`). It has to be
/// passed in rather than written at the call site, since a trait can only be implemented in one
/// block. Items belonging to no trait, such as `new`, go into a separate inherent impl.
///
/// The first form is the goal: every name and number comes from outside, so the skill knows
/// nothing of difficulty or json, and `$params` must carry `cost`, `duration` and `frames`. The
/// second form exists only for bosses whose data is not transcribed yet and builds its numbers
/// with `Params::of(difficulty)`.
///
/// Forms are told apart by the token after `$name` and the one after `$skill_id`. A fragment
/// matcher such as `$x:ty` that fails to parse is a hard error rather than a fallthrough, so
/// every branch must be decided on a literal token ahead of any fragment.
///
/// This file is copied into `core` without the `core::` to `crate::` rewrite, so paths cannot
/// be fully qualified here. `Boss`, `Character`, `CharacterOps`, `SkillMeta`, `SkillOps`,
/// `SkillType` and `NonNull` must all be in scope at the call site.
#[macro_export]
macro_rules! create_boss_skill {
    (
        $name:ident,
        params: $params:ty,
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
            params: $params,
        }

        impl $name {
            pub fn new(
                boss: &Boss,
                skill_mask_offset: usize,
                name: String,
                params: $params,
            ) -> Self {
                Self {
                    parent: NonNull::from_ref(boss),
                    skill_offset: skill_mask_offset,
                    id: (boss.id(), $skill_id),
                    name,
                    params,
                }
            }
        }

        impl SkillMeta for $name {
            fn name(&self) -> &str {
                &self.name
            }

            fn owner(&self) -> Character<'_> {
                unsafe { Character::Boss(self.parent.as_ref()) }
            }

            fn cost(&self) -> u8 {
                self.params.cost
            }

            fn duration(&self) -> u16 {
                self.params.duration
            }

            fn frames(&self) -> u16 {
                self.params.frames
            }

            fn skill_mask_offset(&self) -> usize {
                self.skill_offset
            }

            fn skill_type(&self) -> SkillType {
                $skill_type
            }
        }

        impl SkillOps for $name {
            $($rest)*
        }
    };

    (
        $name:ident,
        $cost:literal,
        $duration:expr,
        $frames:expr,
        $skill_type:path,
        $skill_id:literal,
        params: $params:ty,
        { $($rest:tt)* }
    ) => {
        #[derive(Debug)]
        pub struct $name {
            parent: NonNull<Boss>,
            skill_offset: usize,
            id: (u32, u8),
            name: String,
            params: $params,
        }

        impl $name {
            pub fn new(boss: &Boss, skill_mask_offset: usize) -> Self {
                Self {
                    parent: NonNull::from_ref(boss),
                    skill_offset: skill_mask_offset,
                    id: (boss.id(), $skill_id),
                    name: boss.stats.name.to_string(),
                    params: <$params>::of(boss.stats.difficulty),
                }
            }
        }

        $crate::create_boss_skill!(@ops $name, $cost, $duration, $frames, $skill_type, { $($rest)* });
    };

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

        $crate::create_boss_skill!(@ops $name, $cost, $duration, $frames, $skill_type, { $($rest)* });
    };

    (@ops $name:ident, $cost:literal, $duration:expr, $frames:expr, $skill_type:path, { $($rest:tt)* }) => {
        impl SkillMeta for $name {
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
        }

        impl SkillOps for $name {
            $($rest)*
        }
    };
}
