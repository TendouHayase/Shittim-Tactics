use super::state::KeiState;
use core::{
    effect::{EffectKind, EffectTiming},
    skill::{
        Skill, SkillEffect, SkillEffectTarget, SkillHeader, SkillMeta, SkillParams, SkillType,
    },
    stat::StatKind,
    state::{RemainedEffects, StateData},
    uid::Uid,
    utils::is_inside,
};

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
    pub fn new(owner: Uid, name: &str, skill_offset: usize, params: params::ExParams) -> Self {
        Self {
            header: SkillHeader {
                owner,
                owner_offset: 0,
                name: name.to_string(),
                skill_offset,
                skill_type: SkillType::Ex,
                cost: params.cost(),
                duration: params.duration(),
                frames: params.frames(),
            },
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
    fn skill_effects(&self) -> Vec<core::skill::SkillEffect> {
        let effective_buff = EffectKind::Buff {
            ty: StatKind::MysticEffectiveness,
            duration: self.params.duration,
            scale: self.params.effective_buff_scale,
            amount: 0,
        };

        let atk_buff = EffectKind::Buff {
            ty: StatKind::Atk,
            duration: self.params.duration,
            scale: self.params.atk_buff_scale,
            amount: 0,
        };

        vec![SkillEffect {
            id: self.id(),
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: self.duration(),
            },
            targets: vec![
                SkillEffectTarget::Oneself {
                    kind: effective_buff,
                },
                SkillEffectTarget::Oneself { kind: atk_buff },
                SkillEffectTarget::Student {
                    kind: effective_buff,
                    count: self.params.ally_count,
                },
                SkillEffectTarget::Student {
                    kind: atk_buff,
                    count: self.params.ally_count,
                },
            ],
        }]
    }

    fn apply(&self, caster: &mut StateData, targets: &mut [&mut StateData]) {
        let caster_coord = caster.coordinate();
        let bit = 0x01u64 << self.skill_offset();

        for target in targets.iter_mut() {
            if is_inside(target.coordinate(), self.params.region, caster_coord)
                && (target.effects().0 & bit) == 0
            {
                target.remained_effects_mut().push(RemainedEffects {
                    ticks: self.duration(),
                    offset: self.skill_offset() as u8,
                });

                let effects = target.effects().0 | bit;
                *target.effect_mut() = effects.into();
            }
        }

        if (caster.effects().0 & bit) == 0 {
            caster.remained_effects_mut().push(RemainedEffects {
                ticks: self.duration(),
                offset: self.skill_offset() as u8,
            });

            let effects = caster.effects().0 | bit;
            *caster.effect_mut() = effects.into();
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
    pub fn new(owner: Uid, name: &str, skill_offset: usize, params: params::BasicParams) -> Self {
        Self {
            header: SkillHeader {
                owner,
                owner_offset: 1,
                name: name.to_string(),
                skill_offset,
                skill_type: SkillType::Basic,
                cost: params.cost(),
                duration: params.duration(),
                frames: params.frames(),
            },
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
    fn skill_effects(&self) -> Vec<core::skill::SkillEffect> {
        vec![SkillEffect {
            id: self.id(),
            timing: EffectTiming::Instant,
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::Damage {
                    coef_num: self.params.coef_percent,
                    coef_den: params::PERCENT_DEN,
                },
            }],
        }]
    }

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
    pub fn new(owner: Uid, name: &str, skill_offset: usize, params: params::SubParams) -> Self {
        Self {
            header: SkillHeader {
                owner,
                owner_offset: 2,
                name: name.to_string(),
                skill_offset,
                skill_type: SkillType::Sub,
                cost: params.cost(),
                duration: params.duration(),
                frames: params.frames(),
            },
        }
    }

    /// The effect is declared against [`SkillEffectTarget::Boss`], so the boss is the only
    /// target and the caster is Kei herself.
    pub fn effect_apply(
        _skill: &dyn Skill,
        caster: &mut StateData,
        targets: &mut [&mut StateData],
    ) {
        let Some(boss) = targets.first() else {
            return;
        };

        let len = boss.accumulated_damage().len();
        let prior_idx = caster.extra_as::<KeiState>().recording_start_len;

        let mut acc = 0;
        for i in prior_idx..len {
            if let Some(d) = boss.accumulated_damage()[i].damage {
                acc += d.expected_value();
            }
        }

        let ex = caster.extra_as_mut::<KeiState>();
        ex.acc_damage += acc;
        ex.recording_start_len = len;
    }
}

impl Skill for KeiSubSkill {
    fn skill_effects(&self) -> Vec<SkillEffect> {
        vec![SkillEffect {
            id: self.id(),
            timing: EffectTiming::Persistent {
                interval_frames: 0,
                duration_frames: self.duration(),
            },
            targets: vec![SkillEffectTarget::Boss {
                kind: EffectKind::new_other(Self::effect_apply),
            }],
        }]
    }

    fn apply(&self, caster: &mut StateData, _targets: &mut [&mut StateData]) {
        caster.extra_as_mut::<KeiState>().acc_damage = 0;
    }
}
