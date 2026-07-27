use crate::character::Character;
use crate::state::{State, StateData};
use crate::types::AttackType;
use crate::utils::Position;
use crate::variant_accessor;
use macros::{unreachable_impl_for_empty, EnumAccessors};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Weak;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuffType {
    Atk,
    Crit,
    CritDmg,
    Effectiveness(AttackType),
    BasicProficiency,
    ExSkillDmgDealt,
    DmgDealt,
    Def,
    CostRecovery,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebuffType {
    Atk,
    Crit,
    CritDmg,
    Effectiveness(AttackType),
    ExSkillDmgDealt,
    BasicProficiency,
    DmgDealt,
    Def,
    CostRecovery,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectTiming {
    Instant,
    Persistent {
        interval_frames: u16,
        duration_frames: u16,
    },
}
/// 적용된 스킬 또는 상태효과의 종류를 나타냅니다.
///
/// # Warning
///
/// `Other` 변형의 함수 포인터 주소를 기준으로 `Eq`와 `Hash`를 비교합니다.
/// 컴파일 시 소스코드 상에선 다르더라도 생성되는 기계어가 같으면 같다고 취급될 수 있지만,
/// 로직상 `Other`의 함수는 같은 기능을 하면 같은 것이므로 이 위험을 배제합니다.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EffectKindInner {
    Damage,
    Heal,
    Buff {
        ty: BuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Debuff {
        ty: DebuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    Move,
    Other(*const u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectKind(EffectKindInner);
macro_rules! effect_kind_field {
    ($variant:ident, $name:ident) => {
        paste::paste! {
            impl EffectKind {
                #[inline]
                pub fn [< new_ $name >] () -> Self {
                    Self(EffectKindInner::$variant)
                }

                #[inline] pub fn [<is_ $name>](&self) -> bool {
                    matches!(self.0, EffectKindInner::$variant)
                }
            }
        }
    };

    ($variant:ident { $($field:ident : $ty:ty),+ $(,)? }, $name:ident) => {
        paste::paste! {
            impl EffectKind {
                #[inline]
                pub fn [<new_ $name>] ($($field : $ty),+) -> Self {
                    Self(EffectKindInner::$variant { $($field),+})
                }

                #[inline]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self.0, EffectKindInner::$variant { .. })
                }

                #[inline]
                pub fn [<as_ $name>](&self) -> Option <($($ty),+)> {
                    match self.0 {
                            EffectKindInner::$variant { $($field),+ } => Some(($($field),+)),
                            _ => None,
                        }
                    }

                $(
                    #[inline]
                    pub fn [<as_ $name _ $field >] (& self) -> Option<$ty> {
                        match self.0 {
                            EffectKindInner::$variant { $field, .. } => Some($field), _ => None, } })+ }
                }
    };
}
effect_kind_field!(Damage, damage);
effect_kind_field!(Heal, heal);
effect_kind_field!(
    Buff {
        ty: BuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    buff
);
effect_kind_field!(
    Debuff {
        ty: DebuffType,
        duration: u16,
        scale: u16,
        amount: u32,
    },
    debuff
);
effect_kind_field!(Move, mov);
impl EffectKind {
    #[inline]
    pub fn new_other<'a>(func: fn(&Skill, State<'a>) -> State<'a>) -> Self {
        EffectKind(EffectKindInner::Other(func as *const u8))
    }
    #[inline]
    pub fn is_other(&self) -> bool {
        if let EffectKindInner::Other(_) = self.0 {
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn as_other<'a>(&self) -> Option<fn(&Skill, State<'a>) -> State<'a>> {
        match self.0 {
            EffectKindInner::Other(ptr) => unsafe {
                if ptr.is_null() {
                    None
                } else {
                    std::mem::transmute(ptr)
                }
            },
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Effect<'a> {
    pub name: &'a str,
    pub kind: EffectKind,
    pub timing: EffectTiming,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillEffectTarget {
    Boss { kind: EffectKind },
    Student { kind: EffectKind, count: u8 },
    Land { kind: EffectKind, region: Region },
    Oneself { kind: EffectKind },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Polygon {
        vertex: [Position; 4],
        count: u8,
    },
    Arc {
        radius: u16,
        start_angle_degree: u16,
        end_angle_degree: u16,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillEffect {
    pub id: (u32, u8),
    pub timing: EffectTiming,
    pub targets: Vec<SkillEffectTarget>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Ex,
    Basic,
    Enhanced,
    Sub,
    NormalAttack,
}
use macros::dispatch_method;

pub trait SkillOps {
    fn name(&self) -> &str;
    fn owner(&self) -> Character<'_>;
    fn cost(&self) -> u8;
    fn duration(&self) -> u16;
    fn frames(&self) -> u16;
    fn skill_mask_offset(&self) -> usize;
    fn skill_type(&self) -> SkillType;
    fn skill_effects(&self) -> Vec<SkillEffect>;
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'c mut StateData<'a>,
        targets: &'b mut [&'c mut StateData<'a>],
    );
}

macro_rules! define_skill {
    ($($skill_name:tt),* $(,)?) => {
        #[derive(Debug)]
        pub enum Skill {
            $($skill_name($skill_name),)*
        }

        impl SkillOps for Skill {
            dispatch_method!(Skill, fn name(&self) -> &str, $($skill_name),*);
            dispatch_method!(Skill, fn owner(&self) -> Character<'_>, $($skill_name),*);
            dispatch_method!(Skill, fn cost(&self) -> u8, $($skill_name),*);
            dispatch_method!(Skill, fn duration(&self) -> u16, $($skill_name),*);
            dispatch_method!(Skill, fn frames(&self) -> u16, $($skill_name),*);
            dispatch_method!(Skill, fn skill_mask_offset(&self) -> usize, $($skill_name),*);
            dispatch_method!(Skill, fn skill_type(&self) -> SkillType, $($skill_name),*);
            dispatch_method!(Skill, fn skill_effects(&self) -> Vec<SkillEffect>, $($skill_name),*);
            dispatch_method!(Skill, fn apply<'a: 'b, 'b, 'c: 'b>(&self, caster: &'c mut StateData<'a>, targets: &'b mut [&'c mut StateData<'a>]),  $($skill_name),*);
        }

        // Skill은 생성이 끝난 뒤 내부 데이터 변경이 불가
        unsafe impl Sync for Skill {}
    };
}

// === xtask gen-skills: generated below, do not edit by hand ===
use crate::skills::binah::{
    BinahAtsilutsLight, BinahFireofSeverity2, BinahFiresofSeverity1, BinahPurifyingStorm,
};
use crate::skills::goz::{GozNowYouSeeUs, GozThreeLightMonte};
use crate::skills::kei::{KeiBasicSkill, KeiExSkill, KeiSubSkill};

define_skill!(
    BinahAtsilutsLight,
    BinahFireofSeverity2,
    BinahFiresofSeverity1,
    BinahPurifyingStorm,
    GozNowYouSeeUs,
    GozThreeLightMonte,
    KeiBasicSkill,
    KeiExSkill,
    KeiSubSkill
);
