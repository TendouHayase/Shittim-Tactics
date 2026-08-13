use crate::{
    student::{LevelStats, RawStats, StarCurves, UniqueWeapon},
    utils::{lerp, ols},
};

/// 성급·레벨·능력 개방·전용 무기를 반영한 최종 표시 스탯.
///
/// 1성 기준 직선을 복원해 성급 계수를 곱하지 않고, 해당 성급의 1레벨·90레벨 관측을 바로
/// 보간함. 성급 배수가 상수이고 보간이 선형이라 `lerp(v₁·s, v₉₀·s, x)`와
/// `s · lerp(v₁, v₉₀, x)`가 같기 때문임. 덕분에 성급 계수 표가 필요 없음.
///
/// 능력 개방은 단계당 **1성** 90레벨 값의 0.2%임. `lvl90[0]`의 색인 0이 그것이고 현재 성급이
/// 아님.
///
/// 장비 3종은 여기 없음. `Student::from_file`이 `BaseStats`에 따로 얹음.
///
/// `def`는 성급 배수가 없는 것으로 보이나 성급별 관측을 그대로 쓰므로 확인할 필요가 없음.
/// 나머지 세 스탯과 모양을 맞추려고 `lerp`을 거침.
///
/// 반올림은 모든 항을 더한 뒤 마지막에 한 번만 하고 `round`를 씀. `ceil`을 쓰면 안 됨 —
/// `1.12` 같은 계수가 f64로 정확히 표현되지 않아 참값이 정수여도 `25 * 1.12`가
/// `28.000000000000004`이 되고 `ceil`이 29로 올려버림. `round`는 ±0.5 여유가 있어 ULP 잡음을
/// 삼킴.
///
/// 적용 순서는 아직 검증되지 않음. 사이트 값과 맞춰본 것이 능력 개방과 전용 무기가 모두 0인
/// 경우뿐이라, 전부 더한 뒤 한 번 접는 지금 방식과 항마다 접는 방식이 구분되지 않음. 둘 중
/// 하나가 낀 관측 하나가 있어야 갈림.
///
/// 전용 무기는 5개 관측을 `ols`로 회귀함. 이 5개도 `round(참값)`이라 최소제곱이 관측을
/// 재현한다는 보장이 없고, 실제 잔차가 ±0.485까지 나오므로 반올림 경계에서 1이 틀릴 수 있음.
///
/// `lvl`이 1~90 밖이면 `None`.
///
/// # Panics
///
/// `star`가 1~5 밖이면 패닉함. `star - 1`을 배열 색인으로 쓰므로 `0`은 뺄셈에서, `6` 이상은
/// 색인에서 터짐.
pub fn calcul_stat(
    star: u8,
    lvl: u8,
    unique_weapon_lvl: u8,
    talent: [u8; 3],
    unique_weapon: UniqueWeapon,
    star_curve: StarCurves,
) -> Option<RawStats> {
    // unique_weapon의 hp, atk 배열의 각 인덱스의 레벨 : [1,30,40,50,60]
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
    )?; // 방어력은 성급과 상관없이 레벨만 따지지만 코드 통일성을 위해 `lerp` 사용

    Some(RawStats {
        hp,
        atk,
        def,
        healing,
    })
}
