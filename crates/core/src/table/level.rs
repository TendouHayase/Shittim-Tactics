use crate::{
    student::{LevelStats, RawStats, StarCurves, UniqueWeapon},
    utils::{lerp, ols},
};

/// Level, star tier, talent and unique weapon, before gear and before rounding.
///
/// The star tier's own level 1 and level 90 observations are interpolated directly instead of
/// recovering a 1-star line and scaling it. A star multiplier is constant and interpolation is
/// linear, so the two agree, and this way needs no multiplier table.
///
/// Talent adds 0.2% of the **1-star** level 90 value per rank, which is why the index into
/// `lvl90` is 0 rather than the current tier.
///
/// The unique weapon is a least-squares fit over five observations. Those are rounded too, so
/// the fit is not guaranteed to reproduce them; residuals reach ±0.485 and a value at a
/// rounding boundary can come out one off.
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
    // unique_weapon의 hp, atk 배열이 대응하는 레벨
    let (unique_weapon_hp_delta, unique_weapon_hp_bias) =
        ols(&[1, 30, 40, 50, 60], &unique_weapon.hp);
    let (unique_weapon_atk_delta, unique_weapon_atk_bias) =
        ols(&[1, 30, 40, 50, 60], &unique_weapon.atk);

    let mut hp: f64 = lerp(
        star_curve.hp.lvl1[star as usize - 1] as f64,
        star_curve.hp.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?; // 레벨
    hp += star_curve.hp.lvl90[0] as f64 * 0.002 * talent[0] as f64; // 능력 개방
    hp += if unique_weapon_lvl == 0 {
        0.0
    } else {
        unique_weapon_hp_bias + unique_weapon_hp_delta * unique_weapon_lvl as f64
    }; // 전무 스탯

    let mut atk: f64 = lerp(
        star_curve.atk.lvl1[star as usize - 1] as f64,
        star_curve.atk.lvl90[star as usize - 1] as f64,
        lvl.into(),
        90,
    )?; // 레벨

    atk += star_curve.atk.lvl90[0] as f64 * 0.002 * talent[1] as f64; // 능력개방
    atk += if unique_weapon_lvl == 0 {
        0.0
    } else {
        unique_weapon_atk_bias + unique_weapon_atk_delta * unique_weapon_lvl as f64
    }; // 전무

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
