use crate::{
    student::{RawStats, StarCurves, UniqueWeapon},
    utils::lerp,
};

/// Levels the unique weapon's `hp` and `atk` arrays are sampled at, one per star tier cap.
const UNIQUE_WEAPON_LEVELS: [u8; UniqueWeapon::MAX_STAR as usize + 1] = [1, 30, 40, 50, 60];

/// Interpolates between the two samples bracketing `lvl`, so the five observations come back
/// exactly.
///
/// A single fitted line does not: at level 60 least squares lands about 0.3 below the recorded
/// value, which is enough to move the displayed stat by one.
fn unique_weapon_stat(values: &[u32; UniqueWeapon::MAX_STAR as usize + 1], lvl: u8) -> f64 {
    if lvl == 0 {
        return 0.0;
    }

    let i = UNIQUE_WEAPON_LEVELS
        .iter()
        .rposition(|&sample| sample <= lvl)
        .unwrap_or(0)
        .min(UNIQUE_WEAPON_LEVELS.len() - 2);

    let (x0, x1) = (
        UNIQUE_WEAPON_LEVELS[i] as f64,
        UNIQUE_WEAPON_LEVELS[i + 1] as f64,
    );
    let (y0, y1) = (values[i] as f64, values[i + 1] as f64);

    y0 + (y1 - y0) * (lvl as f64 - x0) / (x1 - x0)
}

/// Level, star tier, talent and unique weapon, before gear and before rounding.
///
/// The star tier's own level 1 and level 90 observations are interpolated directly instead of
/// recovering a 1-star line and scaling it. A star multiplier is constant and interpolation is
/// linear, so the two agree, and this way needs no multiplier table.
///
/// Talent adds 0.2% of the **1-star** level 90 value per rank, which is why the index into
/// `lvl90` is 0 rather than the current tier.
///
/// `def` appears to have no star multiplier, but per-tier observations are used as they are, so
/// it goes through `lerp` like the rest for symmetry.
///
/// `None` when `lvl` is outside 1..=90.
///
/// # Panics
///
/// When `star` is outside 1..=5. `star - 1` indexes an array, so 0 underflows and 6 is out of
/// bounds.
pub fn calcul_stat(
    star: u8,
    lvl: u8,
    unique_weapon_lvl: u8,
    talent: [u8; 3],
    unique_weapon: UniqueWeapon,
    star_curve: StarCurves,
) -> Option<RawStats> {
    let mut hp: f64 = lerp(
        star_curve.hp.lvl1[star as usize - 1] as f64,
        star_curve.hp.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?; // 레벨
    hp += star_curve.hp.lvl90[0] as f64 * 0.002 * talent[0] as f64; // 능력 개방
    hp += unique_weapon_stat(&unique_weapon.hp, unique_weapon_lvl); // 전무

    let mut atk: f64 = lerp(
        star_curve.atk.lvl1[star as usize - 1] as f64,
        star_curve.atk.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?; // 레벨

    atk += star_curve.atk.lvl90[0] as f64 * 0.002 * talent[1] as f64; // 능력개방
    atk += unique_weapon_stat(&unique_weapon.atk, unique_weapon_lvl); // 전무

    let mut healing = lerp(
        star_curve.healing.lvl1[star as usize - 1] as f64,
        star_curve.healing.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?; // 레벨
    healing += star_curve.healing.lvl90[0] as f64 * 0.002 * talent[2] as f64; // 능력개방

    let def = lerp(
        star_curve.def.lvl1[star as usize - 1] as f64,
        star_curve.def.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?;

    Some(RawStats {
        hp,
        atk,
        def,
        healing,
    })
}
