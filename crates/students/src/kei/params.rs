use core::{
    locale::LocalizedName,
    skill::{Region, SkillParams},
};
use serde::Deserialize;

/// Coefficients are all percentages, so the denominator is fixed.
pub const PERCENT_DEN: u16 = 100;
pub const ACC_DAMAGE_CAP_PERCENT: u16 = 5000;
pub const STORED_DAMAGE_PERCENT: u16 = 10;

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
    /// Per ten thousand.
    pub atk_buff_scale: u16,
    /// Added to the effectiveness stat, per ten thousand.
    pub effectiveness_buff_amount: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct BasicParams {
    pub frames: u16,
    pub coef_percent: u16,
    /// Share of the stored damage dealt on top of `coef_percent`.
    pub acc_coef_percent: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct SubParams {
    pub duration: u16,
    /// Buff targets excluding the caster.
    pub ally_count: u8,
    /// Per ten thousand.
    pub crit_buff_scale: u16,
}

/// The `skills` object of `data/students/tendou_kei.json`, before a skill level is chosen.
#[derive(Debug, Deserialize)]
pub struct RawSkills {
    #[serde(rename = "Ex")]
    pub ex: RawEx,
    #[serde(rename = "Basic")]
    pub basic: RawBasic,
    #[serde(rename = "Sub")]
    pub sub: RawSub,
}

// json의 26.8은 부동소수점이라 곱한 뒤 반올림해야 2680이 된다.
fn per_ten_thousand(percent: f64) -> u32 {
    (percent * 100.0).round() as u32
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
            atk_buff_scale: per_ten_thousand(*self.buff.atk.scale.get(i)?) as u16,
            effectiveness_buff_amount: per_ten_thousand(
                *self.buff.mystic_effectiveness.scale.get(i)?,
            ),
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
    #[serde(rename = "acculmulated_damage_coefficient")]
    pub accumulated_coefficient: Vec<u16>,
}

impl RawBasic {
    pub fn pick(&self, level: u8) -> Option<BasicParams> {
        let i = (level as usize).checked_sub(1)?;

        Some(BasicParams {
            frames: self.frames,
            coef_percent: *self.damage.coefficient.get(i)?,
            acc_coef_percent: *self.damage.accumulated_coefficient.get(i)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RawSub {
    pub name: LocalizedName,
    pub duration: u16,
    pub buff: RawSubBuff,
}

#[derive(Debug, Deserialize)]
pub struct RawSubBuff {
    pub crit: RawBuff,
}

impl RawSub {
    pub fn pick(&self, level: u8) -> Option<SubParams> {
        let i = (level as usize).checked_sub(1)?;

        Some(SubParams {
            duration: self.duration,
            ally_count: ON_FIELD_COUNT - 1,
            crit_buff_scale: per_ten_thousand(*self.buff.crit.scale.get(i)?) as u16,
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
