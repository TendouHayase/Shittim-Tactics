use core::skill::SkillParams;

/// The three numbers a skill header carries.
///
/// Passed as a struct rather than three positional arguments so that the two `u16`s cannot be
/// transposed at the call site without the compiler noticing.
#[derive(Debug, Clone, Copy, Default)]
pub struct SkillNumbers {
    pub cost: u8,
    pub duration: u16,
    pub frames: u16,
}

impl SkillNumbers {
    pub fn of<P: SkillParams>(params: &P) -> Self {
        Self {
            cost: params.cost(),
            duration: params.duration(),
            frames: params.frames(),
        }
    }
}

/// Declares a boss skill struct together with its `SkillMeta` and `Skill` implementations.
///
/// ```ignore
/// create_boss_skill!(
///     Name, params: <Params>, SkillType::Ex, 0,
///     effects(id, params) { /* Vec<SkillEffect> */ },
///     { /* Skill methods */ }
/// );
/// ```
///
/// The `effects` block runs once inside `new` and its result is stored in the header. The two
/// names in parentheses are bound for the block: the skill id and a reference to the params.
/// They are named at the call site because a name the macro invents would be invisible to the
/// block under hygiene.
///
/// The block is the body of a closure, so `return` yields the list rather than returning from
/// `new`, and a `todo!()` in it does not turn the rest of `new` into unreachable code.
///
/// The trailing block holds the `Skill` methods (`apply`). It has to be passed in rather than
/// written at the call site, since a trait can only be implemented in one block. Items belonging
/// to no trait go into a separate inherent impl.
///
/// No number appears here. `cost`, `duration` and `frames` arrive as [`SkillNumbers`] and the
/// rest as `$params`, both from whoever loads the boss; a skill with no numbers of its own takes
/// `()`.
///
/// `SkillType` must be in scope at the call site, since the variant is passed in as a path.
#[macro_export]
macro_rules! create_boss_skill {
    (
        $name:ident,
        params: $params:ty,
        $skill_type:path,
        $skill_id:literal,
        effects($id:ident, $p:ident) $effects:block,
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
                numbers: $crate::macros::SkillNumbers,
                params: $params,
            ) -> Self {
                let effects = (|$id: (::core::uid::Uid, usize),
                                $p: &$params|
                 -> Vec<::core::skill::SkillEffect> { $effects })(
                    (owner, $skill_id),
                    &params,
                );

                Self {
                    header: core::skill::SkillHeader {
                        owner,
                        owner_offset: $skill_id,
                        name,
                        skill_offset,
                        skill_type: $skill_type,
                        effects,
                        cost: numbers.cost,
                        duration: numbers.duration,
                        frames: numbers.frames,
                    },
                    params,
                }
            }
        }

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
