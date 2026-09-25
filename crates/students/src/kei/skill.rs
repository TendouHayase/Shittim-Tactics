use error::Error;

use crate::kei::skill::params::RawExBuff;

use super::state::KeiState;
use core::{
    effect::{
        BuffKind::{self, Atk, MysticEffectiveness},
        Effect, EffectTiming,
    },
    simulator::Simulator,
    skill::{
        Skill, SkillEffect, SkillEffectTarget, SkillHeader, SkillMeta, SkillParams, SkillType,
    },
    state::{RemainedEffects, StateData},
    uid::Uid,
    utils::is_inside,
};
use std::sync::Weak;

/// Skill numbers not yet in json.
pub mod params {
    use core::{
        locale::LocalizedName,
        skill::{Region, SkillParams},
    };
    use serde::Deserialize;

    /// Coefficients are all percentages, so the denominator is fixed.
    pub const PERCENT_DEN: u16 = 100;
    pub const ACC_DAMAGE_CAP_PERCENT: u16 = 5000;

    /// On-field capacity, which bounds how many allies the EX buff reaches.
    pub const ON_FIELD_COUNT: u8 = 4;

    #[derive(Debug, Clone, Copy)]
    pub struct ExParams {
        pub cost: u8,
        pub duration: u16,
        pub frames: u16,
        pub region: Region,
        /// Buff targets excluding the caster.
        pub ally_count: u8,
        pub atk_buff_scale: u16,
        /// 83.8, rounded.
        pub effective_buff_scale: u16,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct BasicParams {
        pub frames: u16,
        pub coef_percent: u16,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct SubParams {
        pub duration: u16,
    }

    /// The `skills` object of `data/students/kei.json`, before a skill level is chosen.
    #[derive(Debug, Deserialize)]
    pub struct RawSkills {
        #[serde(rename = "Ex")]
        pub ex: RawEx,
        #[serde(rename = "Basic")]
        pub basic: RawBasic,
        #[serde(rename = "Sub")]
        pub sub: RawSub,
    }

    /// One buffed stat across skill levels. Percentages, so `26.8` means `+26.8%`.
    #[derive(Debug, Deserialize)]
    pub struct RawBuff {
        pub amount: Vec<f64>,
        pub scale: Vec<f64>,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawEx {
        pub name: LocalizedName,
        pub radius: u16,
        pub cost: u8,
        pub frames: u16,
        pub duration: u16,
        pub buff: RawExBuff,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawExBuff {
        pub atk: RawBuff,
        pub mystic_effectiveness: RawBuff,
    }

    impl RawEx {
        pub fn pick(&self, level: u8) -> Option<ExParams> {
            let i = (level as usize).checked_sub(1)?;

            Some(ExParams {
                cost: self.cost,
                duration: self.duration,
                frames: self.frames,
                region: Region::Arc {
                    radius: self.radius,
                    start_angle_degree: 0,
                    end_angle_degree: 360,
                },
                ally_count: ON_FIELD_COUNT - 1,
                atk_buff_scale: self.buff.atk.scale.get(i)?.round() as u16,
                effective_buff_scale: self.buff.mystic_effectiveness.scale.get(i)?.round() as u16,
            })
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct RawBasic {
        pub name: LocalizedName,
        pub frames: u16,
        pub damage: RawBasicDamage,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawBasicDamage {
        pub coefficient: Vec<u16>,
    }

    impl RawBasic {
        pub fn pick(&self, level: u8) -> Option<BasicParams> {
            let i = (level as usize).checked_sub(1)?;

            Some(BasicParams {
                frames: self.frames,
                coef_percent: *self.damage.coefficient.get(i)?,
            })
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct RawSub {
        pub name: LocalizedName,
        pub duration: u16,
    }

    impl RawSub {
        pub fn pick(&self, level: u8) -> Option<SubParams> {
            (level > 0).then_some(SubParams {
                duration: self.duration,
            })
        }
    }

    impl SkillParams for ExParams {
        fn cost(&self) -> u8 {
            self.cost
        }

        fn duration(&self) -> u16 {
            self.duration
        }

        fn frames(&self) -> u16 {
            self.frames
        }
    }

    impl SkillParams for BasicParams {
        fn frames(&self) -> u16 {
            self.frames
        }
    }

    impl SkillParams for SubParams {
        fn duration(&self) -> u16 {
            self.duration
        }
    }
}

/// 증폭 장치를 설치하여 원형범위 내에 있는 아군의 공격력 26.8 → 51% 증가,
/// 신비 특효 44.1 → 83.8% 가산 (25초간)
#[derive(Debug)]
pub struct KeiExSkill {
    header: SkillHeader,
    params: params::ExParams,
}

impl KeiExSkill {
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
                name.to_string(),
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

impl SkillMeta for KeiExSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl Skill for KeiExSkill {
    fn apply(&self, caster: &mut StateData, targets: &mut [&mut StateData]) {
        let caster_coord = caster.coordinate();
        let source = self.skill_offset() as u8;
        let active = |data: &StateData| {
            data.remained_effects()
                .iter()
                .any(|remained| remained.source == source)
        };
        let remained = |effect: Effect| RemainedEffects {
            ticks: self.duration(),
            effect,
            source,
        };

        // 공버프와 신비특효 버프 추가
        for target in targets.iter_mut() {
            if is_inside(target.coordinate(), self.params.region, caster_coord) && !active(target) {
                for skill_effect in self.skill_effects().iter() {
                    if let SkillEffectTarget::Student { .. } = skill_effect.targets {
                        target.remained_effects_mut().push(remained(Effect::Buff {
                            ty: Atk,
                            scale: self.params.atk_buff_scale,
                            amount: 0,
                        }));
                        target.remained_effects_mut().push(remained(Effect::Buff {
                            ty: MysticEffectiveness,
                            scale: self.params.effective_buff_scale,
                            amount: 0,
                        }));
                    }
                }
            }
        }
    }
}

/// 증폭 장치 작동 종료 시 적 1인에게 공격력 148 → 281% 대미지
/// 추가로 해당 증폭 장치 저장량의 40 → 100%만큼 대미지
/// (이 대미지는 치명 공격이 발생하지 않으며, 케이의 능력치에 영향받지 않습니다.)
#[derive(Debug)]
pub struct KeiBasicSkill {
    header: SkillHeader,
    params: params::BasicParams,
}

impl KeiBasicSkill {
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
                name.to_string(),
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

impl SkillMeta for KeiBasicSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl Skill for KeiBasicSkill {
    fn apply(&self, _caster: &mut StateData, _targets: &mut [&mut StateData]) {
        todo!()
    }
}

/// 증폭 장치 작동 시작 시 증폭 장치 범위 내의 아군에게 치명 수치 13.1 → 22.3% 증가 (25초간)
/// 증폭 장치 작동 종료 시, 자신을 제외한 아군이 해당 증폭 장치 범위 내에서
/// 적에게 가한 대미지의 10%를 저장 (케이 기본 공격력의 5000%까지)
/// (저장량은 덮어씌워집니다)
#[derive(Debug)]
pub struct KeiSubSkill {
    header: SkillHeader,
}

impl SkillMeta for KeiSubSkill {
    fn header(&self) -> &SkillHeader {
        &self.header
    }
}

impl KeiSubSkill {
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
                name.to_string(),
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
    pub fn effect_apply(skill: &dyn Skill, caster: &mut StateData, targets: &mut [&mut StateData]) {
        // 보스 데미지가 로그가 아니라 분포(DamageDist)가 되어 "기록 시작 이후 구간"을 읽을
        // 방법이 없다. 저장량을 어떻게 셀지는 A-4에서 정한다.
        let boss_state = for state in targets {};
    }
}

impl Skill for KeiSubSkill {
    fn apply(&self, caster: &mut StateData, _targets: &mut [&mut StateData]) {
        caster.extra_as_mut::<KeiState>().acc_damage = 0;
    }
}
