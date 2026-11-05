use crate::kei::state::KeiExtraState;

use super::params;
use core::{
    actions::ActionContext,
    character::Character,
    damage::Damage,
    effect::{BuffKind, Effect, EffectTiming},
    simulator::Simulator,
    skill::{
        Skill, SkillEffect, SkillEffectTarget, SkillHeader, SkillMeta, SkillParams, SkillType,
    },
    state::{PendingCast, PendingHit, RemainedEffects, State},
    uid::Uid,
    utils::is_inside,
};
use std::sync::Weak;

const BASIC_SLOT: usize = 1;
const SUB_SLOT: usize = 2;

/// Sums what `allies` have dealt so far.
///
/// The amplifier stores a share of a damage window that is correlated with the boss's total, which
/// the boss's distribution cannot express. Expected values are exact when damage is deterministic
/// and a mean-field approximation otherwise.
fn dealt_by(state: &State, allies: &[Uid]) -> u64 {
    allies
        .iter()
        .filter_map(|&ally| state.search_uid(ally))
        .map(|data| data.damage_dealt())
        .sum()
}

/// Stores the share of what `allies` dealt while the amplifier ran, then fires the basic skill.
fn end_device(mut state: State, caster: &dyn Character, allies: &[Uid]) -> State {
    let dealt = dealt_by(&state, allies);
    let cap = caster.stats().atk as u64 * params::ACC_DAMAGE_CAP_PERCENT as u64 / 100;

    let extra = state
        .search_uid_mut(caster.uid())
        .expect("kei uid is not found in state")
        .extra_as_mut::<KeiExtraState>();
    let window = dealt.saturating_sub(extra.recording_start);
    extra.acc_damage = (window * params::STORED_DAMAGE_PERCENT as u64 / 100).min(cap);

    let basic = caster.skills()[BASIC_SLOT].as_ref();
    let boss = state.boss().uid();
    state.schedule_cast(PendingCast {
        frame: state.frames().saturating_add(basic.frames()),
        action: ActionContext {
            caster: caster.uid(),
            targets: vec![boss],
            skill: basic.uid(),
        },
    });

    state
}

/// 증폭 장치를 설치하여 원형범위 내에 있는 아군의 공격력 26.8 → 51% 증가,
/// 신비 특효 44.1 → 83.8% 가산 (25초간)
#[derive(Debug)]
pub struct ExSkill {
    header: SkillHeader,
    params: params::ExParams,
}

impl ExSkill {
    pub fn new(
        owner: Uid,
        name: &str,
        skill_offset: usize,
        params: params::ExParams,
        sim: Weak<Simulator>,
    ) -> Self {
        let id = (owner, 0);

        let effective_buff = Effect::Buff {
            ty: BuffKind::MysticEffectiveness,
            scale: params.effective_buff_scale,
            amount: 0,
        };

        let atk_buff = Effect::Buff {
            ty: BuffKind::Atk,
            scale: params.atk_buff_scale,
            amount: 0,
        };

        let timing = EffectTiming::Persistent {
            interval_frames: 0,
            duration_frames: params.duration(),
        };

        // 케이 본인은 버프 대상이 아니다.
        let effects = vec![
            SkillEffect {
                id,
                timing,
                targets: SkillEffectTarget::Student {
                    kind: effective_buff,
                    count: params.ally_count,
                },
            },
            SkillEffect {
                id,
                timing,
                targets: SkillEffectTarget::Student {
                    kind: atk_buff,
                    count: params.ally_count,
                },
            },
        ];

        Self {
            header: SkillHeader::new(
                owner,
                id.1,
                name,
                skill_offset,
                SkillType::Ex,
                effects,
                params.cost(),
                params.duration(),
                params.frames(),
                sim,
            ),
            params,
        }
    }
}

impl SkillMeta for ExSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl Skill for ExSkill {
    fn apply(&self, mut state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State {
        let sub = caster.skills()[SUB_SLOT].as_ref();

        // 재사용하면 작동 중인 이전 장치는 종료 처리된다. 저장 구간이 새로 시작되기 전에 끝낸다.
        if let Some(device_end) = state.remove_cast(sub.uid()) {
            state = end_device(state, caster, &device_end.action.targets);
        }

        let caster_coord = state
            .search_uid(caster.uid())
            .expect("kei uid is not found in state")
            .coordinate();
        let end_frame = state.frames().saturating_add(self.duration());
        let source = self.skill_offset() as u8;

        // 서브 스킬의 치명 버프는 장치 작동 시작에 함께 걸린다.
        let buffs: Vec<Effect> = self
            .skill_effects()
            .iter()
            .chain(sub.skill_effects())
            .filter_map(|skill_effect| match skill_effect.targets {
                SkillEffectTarget::Student { kind, .. } => Some(kind),
                _ => None,
            })
            .collect();

        // targets에는 효과 개수만큼 같은 대상이 반복되어 들어온다.
        let mut allies: Vec<Uid> = Vec::new();
        for target in targets {
            let target_state = state
                .search_uid_mut(target.uid())
                .expect("target uid is not found in state");

            if allies.contains(&target.uid())
                || !is_inside(target_state.coordinate(), self.params.region, caster_coord)
            {
                continue;
            }

            let effects = target_state.remained_effects_mut();
            effects.retain(|effect| effect.source != source);
            effects.extend(buffs.iter().map(|&effect| RemainedEffects {
                end_frame,
                effect,
                source,
            }));
            allies.push(target.uid());
        }

        let recording_start = dealt_by(&state, &allies);
        state
            .search_uid_mut(caster.uid())
            .expect("kei uid is not found in state")
            .extra_as_mut::<KeiExtraState>()
            .recording_start = recording_start;

        state.schedule_cast(PendingCast {
            frame: end_frame,
            action: ActionContext {
                caster: caster.uid(),
                targets: allies,
                skill: sub.uid(),
            },
        });

        state
    }
}

/// 증폭 장치 작동 종료 시 적 1인에게 공격력 148 → 281% 대미지
/// 추가로 해당 증폭 장치 저장량의 40 → 100%만큼 대미지
/// (이 대미지는 치명 공격이 발생하지 않으며, 케이의 능력치에 영향받지 않습니다.)
#[derive(Debug)]
pub struct BasicSkill {
    header: SkillHeader,
    params: params::BasicParams,
}

impl BasicSkill {
    pub fn new(
        owner: Uid,
        name: &str,
        skill_offset: usize,
        params: params::BasicParams,
        sim: Weak<Simulator>,
    ) -> Self {
        let id = (owner, 1);

        let effects = vec![SkillEffect {
            id,
            timing: EffectTiming::Instant,
            targets: SkillEffectTarget::Boss {
                kind: Effect::Damage {
                    coef_num: params.coef_percent,
                    coef_den: params::PERCENT_DEN,
                },
            },
        }];

        Self {
            header: SkillHeader::new(
                owner,
                id.1,
                name,
                skill_offset,
                SkillType::Basic,
                effects,
                params.cost(),
                params.duration(),
                params.frames(),
                sim,
            ),
            params,
        }
    }
}

impl SkillMeta for BasicSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl Skill for BasicSkill {
    fn apply(
        &self,
        mut state: State,
        caster: &dyn Character,
        _targets: &[&dyn Character],
    ) -> State {
        let stored = state
            .search_uid(caster.uid())
            .expect("kei uid is not found in state")
            .extra_as::<KeiExtraState>()
            .acc_damage;
        let damage = stored * self.params.acc_coef_percent as u64 / params::PERCENT_DEN as u64;

        state.schedule_hit(PendingHit {
            frame: state.frames(),
            damage: Damage::point(damage),
            source: self.uid(),
        });

        state
    }
}

/// 증폭 장치 작동 시작 시 증폭 장치 범위 내의 아군에게 치명 수치 13.1 → 22.3% 증가 (25초간)
/// 증폭 장치 작동 종료 시, 자신을 제외한 아군이 해당 증폭 장치 범위 내에서
/// 적에게 가한 대미지의 10%를 저장 (케이 기본 공격력의 5000%까지)
/// (저장량은 덮어씌워집니다)
///
/// The start is applied by [`ExSkill`]; landing this skill is the amplifier ending.
#[derive(Debug)]
pub struct SubSkill {
    header: SkillHeader,
}

impl SkillMeta for SubSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl SubSkill {
    pub fn new(
        owner: Uid,
        name: &str,
        skill_offset: usize,
        params: params::SubParams,
        sim: Weak<Simulator>,
    ) -> Self {
        let id = (owner, 2);

        let effects = vec![SkillEffect {
            id,
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: params.duration(),
            },
            targets: SkillEffectTarget::Student {
                kind: Effect::Buff {
                    ty: BuffKind::Crit,
                    scale: params.crit_buff_scale,
                    amount: 0,
                },
                count: params.ally_count,
            },
        }];

        Self {
            header: SkillHeader::new(
                owner,
                id.1,
                name,
                skill_offset,
                SkillType::Sub,
                effects,
                params.cost(),
                params.duration(),
                params.frames(),
                sim,
            ),
        }
    }
}

impl Skill for SubSkill {
    fn apply(&self, state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State {
        let allies: Vec<Uid> = targets.iter().map(|target| target.uid()).collect();

        end_device(state, caster, &allies)
    }
}
