use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Weak;

use macros::unreachable_impl_for_empty;

use crate::character::Character;
use crate::state::{StateData, Stateful};
use crate::types::AttackType;
use crate::utils::Position;

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
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind<T = (), S = ()>
where
    T: Skill,
    S: for<'a> Stateful<'a>,
{
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
    Other(fn(&T, S) -> S),
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
        // 왼쪽 위부터 시계방향
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
    pub id: (u32, u8), // (캐릭터 id, 스킬 인덱스)
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

#[unreachable_impl_for_empty]
pub trait Skill<T: Send + Sync + Clone + Default = ()>: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn owner(&self) -> Weak<dyn Character>;
    fn cost(&self) -> u8;
    fn frames(&self) -> u16;
    fn duration(&self) -> u16;
    fn skill_mask_offset(&self) -> usize;
    fn skill_type(&self) -> SkillType;
    fn skill_effects<'a>(&'a self) -> Vec<SkillEffect>;
    fn apply<'a: 'b, 'b, 'c: 'b>(
        &self,
        caster: &'b StateData<'a>,
        targets: &'b [&'c StateData<'a>],
    ) -> Vec<StateData<'a>>;
}
