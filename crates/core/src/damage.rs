use std::sync::{Arc, RwLock};

use stochastic::{
    distributions::{IrwinHall, Uniform},
    utils::build_prefix_sum,
};

use crate::{
    skill::{Buff, CasterContext, Debuff, EffectKind, SkillType, TargetContext},
    types::{AttackType, damage_scale, is_weak},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Damage {
    pub normal: Uniform,
    pub crit: Uniform,
    pub crit_num: u32,
    pub crit_den: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DamageCache {
    cached: Option<IrwinHall>,
    last_len: usize,
}

impl Damage {
    pub fn from_skill_context(
        src: &CasterContext,
        tgt: &TargetContext,
        scale_num: u64,
        scale_den: u64,
    ) -> Damage {
        // Since stats are added via multiplication or addition depending on buffs and debuffs,
        // a `scale` variable is required, so each element is copied rather than copying the entire object.

        let mut atk = src.stats.atk;
        let mut stability = src.stats.stability;
        let mut stability_rate = src.stats.stability_rate;
        let mut crit = src.stats.crit;
        let mut crit_dmg = src.stats.crit_dmg;
        let mut dmg_dealt = src.stats.dmg_dealt;
        let mut ex_skill_dmg_dealt = src.stats.ex_skill_dmg_dealt;
        let mut basic_proficiency = src.stats.basics_proficiency;
        let mut explosive_effectiveness = src.stats.explosive_effectiveness;
        let mut piercing_effectiveness = src.stats.piercing_effectiveness;
        let mut corrosive_effectiveness = src.stats.corrosive_effectiveness;
        let mut mystic_effectiveness = src.stats.mystic_effectiveness;
        let mut sonic_effectiveness = src.stats.sonic_effectiveness;

        let mut target_def = tgt.stats.def;

        let mut atk_scale = 100;
        let mut crit_scale = 100;
        let mut crit_dmg_scale = 100;
        let stability_scale = 100;
        let stability_rate_scale = 100;
        let mut dmg_dealt_scale = 100;
        let mut ex_skill_dmg_dealt_scale = 100;
        let mut basic_proficiency_scale = 100;
        let mut explosive_effectiveness_scale = 100;
        let mut piercing_effectiveness_scale = 100;
        let mut corrosive_effectiveness_scale = 100;
        let mut mystic_effectiveness_scale = 100;
        let mut sonic_effectiveness_scale = 100;

        let mut target_def_scale = 100;

        for effect in src.effects.iter() {
            match &effect.kind {
                EffectKind::Buff {
                    ty,
                    duration: _,
                    scale,
                    amount: increase,
                } => match ty {
                    Buff::Atk => {
                        atk += increase;
                        atk_scale += scale;
                    }
                    Buff::Crit => {
                        crit += *increase as u16;
                        crit_scale += scale;
                    }
                    Buff::CritDmg => {
                        crit_dmg += increase;
                        crit_dmg_scale += scale;
                    }
                    Buff::DmgDealt => {
                        dmg_dealt += increase;
                        dmg_dealt_scale += scale;
                    }
                    Buff::ExSkillDmgDealt => {
                        ex_skill_dmg_dealt += increase;
                        ex_skill_dmg_dealt_scale += scale;
                    }
                    Buff::Effectiveness(atk_type) => match atk_type {
                        AttackType::Explosive => {
                            explosive_effectiveness += increase;
                            explosive_effectiveness_scale += scale;
                        }
                        AttackType::Piercing => {
                            piercing_effectiveness += increase;
                            piercing_effectiveness_scale += scale;
                        }
                        AttackType::Corrosive => {
                            corrosive_effectiveness += increase;
                            corrosive_effectiveness_scale += scale;
                        }
                        AttackType::Mystic => {
                            mystic_effectiveness += increase;
                            mystic_effectiveness_scale += scale;
                        }
                        AttackType::Sonic => {
                            sonic_effectiveness += increase;
                            sonic_effectiveness_scale += scale;
                        }
                        AttackType::Normal => (),
                    },
                    Buff::BasicProficiency => {
                        basic_proficiency += increase;
                        basic_proficiency_scale += scale;
                    }
                    _ => (),
                },
                EffectKind::Debuff {
                    ty,
                    duration: _,
                    scale,
                    amount: decrease,
                } => match ty {
                    Debuff::Atk => {
                        atk -= decrease;
                        atk_scale -= scale;
                    }
                    Debuff::Crit => {
                        crit -= *decrease as u16;
                        crit_scale -= scale;
                    }
                    Debuff::CritDmg => {
                        crit_dmg -= decrease;
                        crit_dmg_scale -= scale;
                    }
                    Debuff::DmgDealt => {
                        dmg_dealt -= decrease;
                        dmg_dealt_scale -= scale;
                    }
                    Debuff::ExSkillDmgDealt => {
                        ex_skill_dmg_dealt -= decrease;
                        ex_skill_dmg_dealt_scale -= scale;
                    }
                    Debuff::Effectiveness(atk_type) => match atk_type {
                        AttackType::Explosive => {
                            explosive_effectiveness -= decrease;
                            explosive_effectiveness_scale -= scale;
                        }
                        AttackType::Piercing => {
                            piercing_effectiveness -= decrease;
                            piercing_effectiveness_scale -= scale;
                        }
                        AttackType::Corrosive => {
                            corrosive_effectiveness -= decrease;
                            corrosive_effectiveness_scale -= scale;
                        }
                        AttackType::Mystic => {
                            mystic_effectiveness -= decrease;
                            mystic_effectiveness_scale -= scale;
                        }
                        AttackType::Sonic => {
                            sonic_effectiveness -= decrease;
                            sonic_effectiveness_scale -= scale;
                        }
                        AttackType::Normal => (),
                    },
                    Debuff::BasicProficiency => {
                        basic_proficiency -= decrease;
                        basic_proficiency_scale -= scale;
                    }
                    Debuff::Def => {
                        target_def -= decrease;
                        target_def_scale -= scale;
                    }
                },
                _ => (),
            }
        }

        atk = atk * atk_scale as u32 / 100;
        stability = stability * stability_scale / 100;
        stability_rate = stability_rate * stability_rate_scale / 100;
        crit = crit * crit_scale / 100;
        crit_dmg = crit_dmg * crit_dmg_scale as u32 / 100;
        dmg_dealt = dmg_dealt * dmg_dealt_scale as u32 / 100;
        ex_skill_dmg_dealt = ex_skill_dmg_dealt * ex_skill_dmg_dealt_scale as u32 / 100;
        basic_proficiency = basic_proficiency * basic_proficiency_scale as u32 / 100;
        explosive_effectiveness =
            explosive_effectiveness * explosive_effectiveness_scale as u32 / 100;
        piercing_effectiveness = piercing_effectiveness * piercing_effectiveness_scale as u32 / 100;
        corrosive_effectiveness =
            corrosive_effectiveness * corrosive_effectiveness_scale as u32 / 100;
        mystic_effectiveness = mystic_effectiveness * mystic_effectiveness_scale as u32 / 100;
        sonic_effectiveness = sonic_effectiveness * sonic_effectiveness_scale as u32 / 100;

        let mut max_dmg: u64 = atk_scale as u64 * atk as u64 / 100;

        match src.skill_type {
            SkillType::Ex => max_dmg *= ex_skill_dmg_dealt as u64 / 10000,
            _ => max_dmg *= basic_proficiency as u64 / 10000,
        }

        let final_crit_num = (crit as i32 - tgt.stats.crit_res).max(0) as u32 * 100u32;
        let final_crit_deno = (crit as i32 - tgt.stats.crit_res).max(0) as u32 * 100u32 + 666666;

        let final_crit_dmg_scale: f64 = (crit_dmg - tgt.stats.crit_dmg_res) as f64 / 10000f64;

        let src_atk_type = src.stats.attack_type;

        let mut effectiveness_dmg_scale: u32 = 0;
        if is_weak(src_atk_type, tgt.stats.armor_type) {
            match src_atk_type {
                AttackType::Explosive => effectiveness_dmg_scale = explosive_effectiveness,
                AttackType::Piercing => effectiveness_dmg_scale = piercing_effectiveness,
                AttackType::Mystic => effectiveness_dmg_scale = mystic_effectiveness,
                AttackType::Corrosive => effectiveness_dmg_scale = corrosive_effectiveness,
                AttackType::Sonic => effectiveness_dmg_scale = sonic_effectiveness,
                _ => (),
            }
        }

        effectiveness_dmg_scale =
            effectiveness_dmg_scale / 10000 + damage_scale(&src_atk_type, &tgt.stats.armor_type);

        target_def = target_def * target_def_scale as u32 / 100;

        max_dmg *= effectiveness_dmg_scale as u64;

        max_dmg = max_dmg * 1666 / (target_def - src.stats.defense_piercing as u32 + 1666) as u64;

        max_dmg *= dmg_dealt as u64 / 10000;
        max_dmg -= max_dmg * (tgt.stats.dmg_resist as u64 - 10000) / 10000;

        max_dmg *= scale_num;
        max_dmg /= scale_den;

        let min_dmg: u64 =
            max_dmg - (((stability) / (stability + 1000)) + stability_rate / 5) as u64;

        Damage {
            normal: Uniform {
                min: min_dmg,
                max: max_dmg,
            },
            crit: Uniform {
                min: (min_dmg as f64 * final_crit_dmg_scale) as u64,
                max: (max_dmg as f64 * final_crit_dmg_scale) as u64,
            },
            crit_num: final_crit_num,
            crit_den: final_crit_deno,
        }
    }

    pub fn to_irwin_hall(&self) -> IrwinHall {
        let normal_len = self.normal.max - self.normal.min + 1;
        let crit_len = self.crit.max - self.crit.min + 1;

        let weight_normal = (self.crit_den - self.crit_num) as u128 * crit_len as u128;
        let weight_crit = self.crit_num as u128 * normal_len as u128;

        let lo = self.normal.min.min(self.crit.min);
        let hi = self.normal.max.max(self.crit.max);
        let len = (hi - lo + 1) as usize;

        let mut counts = vec![0u128; len];
        for v in self.normal.min..=self.normal.max {
            counts[(v - lo) as usize] += weight_normal;
        }
        for v in self.crit.min..=self.crit.max {
            counts[(v - lo) as usize] += weight_crit;
        }

        let total_combinations = normal_len as u128 * crit_len as u128;
        let prefix_sum = build_prefix_sum(&counts);

        IrwinHall {
            prefix_sum: Arc::new(prefix_sum),
            uniforms: Arc::new(RwLock::new(vec![])),
            n: 1,
            min: lo,
            max: hi,
            total_combinations,
        }
    }
}

impl DamageCache {
    pub fn get_or_compute(&mut self, history: &[Damage]) -> &IrwinHall {
        if self.cached.is_none() || self.last_len != history.len() {
            let mut acc: Option<IrwinHall> = None;

            for dmg in history {
                let ih = dmg.to_irwin_hall();
                acc = Some(match acc {
                    None => ih,
                    Some(existing) => &existing + &ih,
                })
            }

            self.cached = acc;
            self.last_len = history.len();
        }

        self.cached.as_ref().unwrap()
    }

    pub fn append(&mut self, dmg: &Damage) {
        let ih = dmg.to_irwin_hall();
        match &mut self.cached {
            Some(existing) => *existing += &ih,
            None => self.cached = Some(ih),
        }
    }
}
