/// Declares a boss skill struct together with its `SkillMeta` and `Skill` implementations.
///
/// ```ignore
/// create_boss_skill!(Name, params: <Params>, SkillType::Ex, 0, { /* Skill methods */ });
/// create_boss_skill!(Name, cost, duration, frames, SkillType::Ex, 0, { /* ... */ });
/// create_boss_skill!(Name, cost, duration, frames, SkillType::Ex, 0, params: <Params>, { /* ... */ });
/// ```
///
/// The trailing block holds the `Skill` methods (`skill_effects`, `apply`). It has to be passed
/// in rather than written at the call site, since a trait can only be implemented in one block.
/// Items belonging to no trait go into a separate inherent impl.
///
/// The first form is the goal: every name and number comes from outside, and `$params` must
/// carry `cost`, `duration` and `frames`. The other two take those three as literals, for bosses
/// whose data is not transcribed yet.
///
/// Forms are told apart by the token after `$name` and the one after `$skill_id`. A fragment
/// matcher such as `$x:ty` that fails to parse is a hard error rather than a fallthrough, so
/// every branch must be decided on a literal token ahead of any fragment.
///
/// `SkillType` must be in scope at the call site, since the variant is passed in as a path.
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
            header: ::core::skill::SkillHeader,
            params: $params,
        }

        impl $name {
            pub fn new(
                owner: ::core::uid::Uid,
                skill_offset: usize,
                name: String,
                params: $params,
            ) -> Self {
                Self {
                    header: ::core::skill::SkillHeader {
                        owner,
                        owner_offset: $skill_id,
                        name,
                        skill_offset,
                        skill_type: $skill_type,
                        cost: params.cost,
                        duration: params.duration,
                        frames: params.frames,
                    },
                    params,
                }
            }
        }

        $crate::create_boss_skill!(@impls $name, { $($rest)* });
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
            header: ::core::skill::SkillHeader,
            params: $params,
        }

        impl $name {
            pub fn new(
                owner: ::core::uid::Uid,
                skill_offset: usize,
                name: String,
                params: $params,
            ) -> Self {
                Self {
                    header: ::core::skill::SkillHeader {
                        owner,
                        owner_offset: $skill_id,
                        name,
                        skill_offset,
                        skill_type: $skill_type,
                        cost: $cost,
                        duration: $duration,
                        frames: $frames,
                    },
                    params,
                }
            }
        }

        $crate::create_boss_skill!(@impls $name, { $($rest)* });
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
            header: ::core::skill::SkillHeader,
        }

        impl $name {
            pub fn new(owner: ::core::uid::Uid, skill_offset: usize, name: String) -> Self {
                Self {
                    header: ::core::skill::SkillHeader {
                        owner,
                        owner_offset: $skill_id,
                        name,
                        skill_offset,
                        skill_type: $skill_type,
                        cost: $cost,
                        duration: $duration,
                        frames: $frames,
                    },
                }
            }
        }

        $crate::create_boss_skill!(@impls $name, { $($rest)* });
    };

    (@impls $name:ident, { $($rest:tt)* }) => {
        impl ::core::skill::SkillMeta for $name {
            fn header(&self) -> &::core::skill::SkillHeader {
                &self.header
            }
        }

        impl ::core::skill::Skill for $name {
            $($rest)*
        }
    };
}
