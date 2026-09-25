pub fn apply_def(damage: u64, def: u32, def_piercing: u16) -> u64 {
    damage / (def as u64 - def_piercing as u64 + 1666u64) * 1666u64
}

pub fn crit_rate(crit: u16, crit_res: i32) -> f64 {
    let delta: i32 = crit as i32 - crit_res;
    delta as f64 / (delta as f64 + 666.666)
}

pub fn crit_rate_fraction(crit: u16, crit_res: i32) -> (u32, u32) {
    let delta: u32 = (crit as i32 - crit_res) as u32;
    (delta, (delta + 666))
}

pub fn crit_damage_coefficient(crit_dmg: u32, crit_dmg_res: u32) -> f64 {
    (crit_dmg - crit_dmg_res) as f64 / 10000.0 
}

pub fn stability_coefficient(stability: u16, stability_rate: u16) -> f64 {
    (stability as f64 / (stability as f64 + 1000.0)) + (stability_rate as f64 * 0.2)
}

pub fn delta_cc_time(default_prob: f64, cc_power: u16, cc_res: u16) -> f64 {
    default_prob * (cc_power + 100 - cc_res) as f64 / 100.0
}
