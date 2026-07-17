use error::error::Error;
use std::{fs::File, io::BufReader};

pub fn read_json<'a, T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Error> {
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let data: T = serde_json::from_reader(reader)?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use crate::read_json;

    #[test]
    fn read_test() {
        let data: serde_json::Value = read_json(r"./tests/test.json").unwrap();

        let answer = r#"{"heavy":{"normal":{"BaseStats":{"accuracy":1300,"armor_type":"Heavy","atk":1500,"atk_speed":10000,"attack_type":"Normal","basics_proficiency":10000,"block_ratge_bonus":0,"buff_retention":10000,"cc_power":100,"cc_res":100,"cost_recovery":700,"crit":200,"crit_dmg":200,"crit_res":20,"debuff_retention":10000,"def":1700,"defense_piercing":0,"dmg_dealt":10000,"dmg_resist":10000,"evasion":100,"ex_skill_dmg_dealt":10000,"ex_skill_dmg_resist":10000,"explosive_effectiveness":10000,"healing":1700,"healing_boost":10000,"hp":300000,"level":17,"mag_count":1,"mov_speed":500,"mystic_effectiveness":10000,"normal_attack_range":3000,"piercing_effectiveness":10000,"recovery_boost":10000,"sighting_range":65535,"sonic_effectiveness":10000,"stability":300},"groggy_duration":10,"groggy_gauge":150000,"phase_switching_hp":[180000,75000]}}}"#;

        assert_eq!(data.to_string(), answer.to_string());
    }
}
