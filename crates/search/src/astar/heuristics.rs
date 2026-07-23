use core::{damage::Damage, simulator::Simulator, state::Stateful};

pub fn heuristics<'a>(sim: &impl Simulator, state: &impl Stateful<'a>) -> u64 {
    let students = state.students();
    let boss = state.boss();

    let mut boss_accumulated_damage: Vec<Damage> =
        Vec::with_capacity(boss.accumulated_damage.len());

    for d in &boss.accumulated_damage {
        match d.damage {
            Some(damage) => boss_accumulated_damage.push(damage),
            None => {
                if d.damage.is_none() {
                    continue;
                }
                for _ in 0..d.ticks {
                    boss_accumulated_damage.push(d.damage.unwrap());
                }
            }
        }
    }
    let boss_accumulated_damage_guard = state
        .boss()
        .accumulated_damage_cache
        .get_or_compute(&boss_accumulated_damage);

    let boss_accumulated_damage_dists = boss_accumulated_damage_guard.as_ref().unwrap();

    let remain_boss_max_hp = boss.character.stats().hp - boss_accumulated_damage_dists.min;
    let remain_boss_min_hp = boss.character.stats().hp - boss_accumulated_damage_dists.max;

    if remain_boss_max_hp == 0 {
        return 0;
    }

    let max_damage = (*sim.damage_map()).iter().max().unwrap();
    let mut all_ex_damage = 0;
    let mut max_ex_damage = 0;
    let mut max_damage_cast_frames = 0;

    for student in students {
        let damage = student.damage_with_effects();

        all_ex_damage += damage.unwrap_or_default().crit.max;

        if damage.unwrap_or_default().crit.max > max_ex_damage {
            let frames = student.character.skill_list()[0].duration();

            max_ex_damage = max_ex_damage.max(damage.unwrap_or_default().crit.max / frames as u64);

            max_damage_cast_frames = frames;
        }
    }

    let h_max_dps = remain_boss_min_hp / max_damage.1.normal.max;
    let h_all_ex_dps = remain_boss_min_hp / all_ex_damage;
    let h_min_cast = (remain_boss_min_hp * max_damage_cast_frames as u64) / max_ex_damage;

    h_max_dps.max(h_all_ex_dps).max(h_min_cast)
}
