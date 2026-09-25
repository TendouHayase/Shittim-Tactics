use super::params;
use super::state::KeiState;
use core::{
    character::Character,
    effect::{
        BuffKind::{self, Atk, MysticEffectiveness},
        Effect, EffectTiming,
    },
    simulator::Simulator,
    skill::{
        Skill, SkillEffect, SkillEffectTarget, SkillHeader, SkillMeta, SkillParams, SkillType,
    },
    state::{RemainedEffects, State, StateData},
    uid::Uid,
    utils::is_inside,
};
use std::sync::Weak;

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

        let effects = vec![
            SkillEffect {
                id,
                timing,
                targets: SkillEffectTarget::Oneself {
                    kind: effective_buff,
                },
            },
            SkillEffect {
                id,
                timing,
                targets: SkillEffectTarget::Oneself { kind: atk_buff },
            },
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
        let caster_coord;
        {
            let caster_state = state
                .search_uid_mut(caster.uid())
                .expect("kei uid is not found in state");

            caster_coord = caster_state.coordinate();
        }
        let source = self.skill_offset() as u8;

        let active = |data: &StateData| {
            data.remained_effects()
                .iter()
                .any(|remained| remained.source == source)
        };

        let current_frame = state.frames();

        let remained = |effect: Effect| RemainedEffects {
            end_frame: current_frame + self.duration(),
            effect,
            source,
        };

        // 공버프와 신비특효 버프 추가
        for target in targets {
            let target_state = state
                .search_uid_mut(target.uid())
                .expect("target uid is not found in state");
            if is_inside(target_state.coordinate(), self.params.region, caster_coord)
                && !active(target_state)
            {
                for skill_effect in self.skill_effects().iter() {
                    if let SkillEffectTarget::Student { .. } = skill_effect.targets {
                        target_state
                            .remained_effects_mut()
                            .push(remained(Effect::Buff {
                                ty: Atk,
                                scale: self.params.atk_buff_scale,
                                amount: 0,
                            }));
                        target_state
                            .remained_effects_mut()
                            .push(remained(Effect::Buff {
                                ty: MysticEffectiveness,
                                scale: self.params.effective_buff_scale,
                                amount: 0,
                            }));
                    }
                }
            }
        }

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
    fn apply(&self, mut state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State {
        todo!()
    }
}

/// 증폭 장치 작동 시작 시 증폭 장치 범위 내의 아군에게 치명 수치 13.1 → 22.3% 증가 (25초간)
/// 증폭 장치 작동 종료 시, 자신을 제외한 아군이 해당 증폭 장치 범위 내에서
/// 적에게 가한 대미지의 10%를 저장 (케이 기본 공격력의 5000%까지)
/// (저장량은 덮어씌워집니다)
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
            targets: SkillEffectTarget::Boss {
                kind: Effect::new_other(Self::effect_apply),
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

    /// The effect is declared against [`SkillEffectTarget::Boss`], so the boss is the only
    /// target and the caster is Kei herself.
    pub fn effect_apply(
        _skill: &dyn Skill,
        _caster: &mut StateData,
        targets: &mut [&mut StateData],
    ) {
        // 보스 데미지가 로그가 아니라 분포(DamageDist)가 되어 "기록 시작 이후 구간"을 읽을
        // 방법이 없다. 저장량을 어떻게 셀지는 A-4에서 정한다.
        for _state in targets {}
    }
}

impl Skill for SubSkill {
    fn apply(&self, mut state: State, caster: &dyn Character, targets: &[&dyn Character]) -> State {
        todo!()
    }
}
