use crate::constants::TPS;
use crate::skill::Region;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A decimal from json held without rounding. The denominator is always `10^exp`.
///
/// Numerator and scale are kept apart so that multiplication happens before division: folding
/// through a float would shift stats by tens at a rounding boundary, which changes whether a
/// tactic works.
///
/// serde hands over an `f64`, so the original text is gone. The scale is recovered from the
/// shortest round-tripping representation instead. Game data with a few decimal places restores
/// exactly; values beyond 17 significant digits do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Ratio {
    num: i64,
    exp: u8,
}

impl Ratio {
    /// The largest `exp` for which `10^exp` fits in an `i64`.
    pub const MAX_EXP: u8 = 18;

    /// Strips trailing zeros so that equal values share one representation. Without this,
    /// `2.50` and `2.5` would differ under `Eq` and `Hash`.
    pub fn new(num: i64, exp: u8) -> Self {
        let mut num = num;
        let mut exp = exp.min(Self::MAX_EXP);

        while exp > 0 && num % 10 == 0 {
            num /= 10;
            exp -= 1;
        }

        Self { num, exp }
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        10i64.pow(self.exp as u32)
    }

    /// `value * self` as an integer. Multiplication comes first so nothing is rounded in
    /// between, and the division truncates toward zero. Goes through `i128` because the product
    /// can leave `i64`.
    pub fn apply(self, value: i64) -> i64 {
        ((value as i128 * self.num as i128) / self.den() as i128) as i64
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den() as f64
    }
}

impl Serialize for Ratio {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.exp == 0 {
            self.num.serialize(serializer)
        } else {
            self.to_f64().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;

        if !value.is_finite() {
            return Err(serde::de::Error::custom("ratio must be finite"));
        }

        // `f64`의 `Display`는 지수 표기를 쓰지 않아 항상 `-?\d+(\.\d+)?` 꼴임.
        let text = value.to_string();
        let exp = text.split_once('.').map_or(0, |(_, frac)| frac.len());

        if exp > Self::MAX_EXP as usize {
            return Err(serde::de::Error::custom(format!(
                "ratio has too many decimal places: {text}"
            )));
        }

        let digits: String = text.chars().filter(|c| *c != '.').collect();
        let num = digits
            .parse::<i64>()
            .map_err(|_| serde::de::Error::custom(format!("ratio out of range: {text}")))?;

        Ok(Self::new(num, exp as u8))
    }
}

impl From<Ratio> for f64 {
    fn from(value: Ratio) -> Self {
        value.to_f64()
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.num as i128 * other.den() as i128).cmp(&(other.num() as i128 * self.den() as i128))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Eq, Hash)]
pub struct Position {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

/// `[x, y]` in json. Written by hand to avoid enabling the serde feature of `ordered-float`,
/// and it keeps field names from repeating throughout the data files.
impl Serialize for Position {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (self.x.0, self.y.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <(f32, f32)>::deserialize(deserializer).map(Into::into)
    }
}

impl From<(f32, f32)> for Position {
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: OrderedFloat(value.0),
            y: OrderedFloat(value.1),
        }
    }
}

impl From<(f64, f64)> for Position {
    fn from(value: (f64, f64)) -> Self {
        Self {
            x: OrderedFloat(value.0 as f32),
            y: OrderedFloat(value.1 as f32),
        }
    }
}

impl From<(i32, i32)> for Position {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: OrderedFloat(value.0 as f32),
            y: OrderedFloat(value.1 as f32),
        }
    }
}

impl Add<Position> for Position {
    type Output = Position;
    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Position;
    fn sub(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

use std::ops::{Add, Sub};

#[inline]
pub fn euclidean_distance(lhs: Position, rhs: Position) -> f64 {
    ((lhs.x - rhs.x) * (lhs.x - rhs.x) + (lhs.y - rhs.y) * (lhs.y - rhs.y))
        .sqrt()
        .into()
}

/// Cross product of the vectors `p1 -> p2` and `q1 -> q2`.
#[inline]
pub fn cross_product(p1: Position, p2: Position, q1: Position, q2: Position) -> OrderedFloat<f32> {
    let (px, py) = (p2.x - p1.x, p2.y - p1.y);
    let (qx, qy) = (q2.x - q1.x, q2.y - q1.y);
    px * qy - py * qx
}

#[inline]
pub fn dot_product(p1: Position, p2: Position, q1: Position, q2: Position) -> OrderedFloat<f32> {
    let (px, py) = (p2.x - p1.x, p2.y - p1.y);
    let (qx, qy) = (q2.x - q1.x, q2.y - q1.y);
    px * qx + py * qy
}

pub fn is_inside(p: Position, region: Region, bias: Position) -> bool {
    match region {
        Region::Polygon { vertex, count } => {
            let valid_region: Vec<Position> = vertex
                .iter()
                .enumerate()
                .filter(|i| i.0 < count.into())
                .map(|pos| Position {
                    x: pos.1.x + bias.x,
                    y: pos.1.y + bias.y,
                })
                .collect();

            // 부호 비트 추출
            let sign_bit: i32 =
                if cross_product(valid_region[0], valid_region[1], valid_region[0], p)
                    .is_sign_positive()
                {
                    1
                } else {
                    -1
                };
            let mut is_include = 0;

            for idx in 1..count {
                let s = if cross_product(
                    valid_region[idx as usize],
                    valid_region[(idx as usize + 1) % count as usize],
                    valid_region[idx as usize],
                    p,
                )
                .is_sign_positive()
                {
                    1
                } else {
                    -1
                };

                // 기존 부호와 같은지 비교
                is_include |= sign_bit ^ s;
            }

            is_include == 0
        }
        Region::Arc {
            radius,
            start_angle_degree,
            end_angle_degree,
        } => {
            let relative_p = p - bias;

            let distance = euclidean_distance((0, 0).into(), relative_p);

            if distance > radius as f64 {
                return false;
            }

            // 원점
            let o = (0, 0).into();

            // radius 길이의 y축
            let y_axis: Position = (0, radius as i32).into();

            // |a|*|b|*sin(θ)
            let cross = cross_product(o, y_axis, o, relative_p);

            // |a|*|b|*cos(θ)
            let dot = dot_product(o, y_axis, o, relative_p);

            // atan(sin(θ)/cos(θ))
            let radian = cross.atan2(*dot).rem_euclid(360.0);

            start_angle_degree as f32 <= radian.to_degrees()
                && radian.to_degrees() <= end_angle_degree as f32
        }
    }
}

pub const fn time_to_ticks(time_num: u16, time_den: u16) -> u16 {
    time_num * TPS / time_den
}

/// Linear interpolation giving `start` at `lvl == 1` and `end` at `lvl == lvl_max`.
///
/// One-based, because game levels are: `lvl - 1` is divided by `lvl_max - 1` rather than taking
/// a `t` in `[0, 1]`.
///
/// The result is not rounded. Callers fold to a displayed value themselves, and must do so only
/// once, after every term has been added.
///
/// `None` when `lvl` is 0, when `lvl_max <= 1`, or when `lvl > lvl_max`. Extrapolation is
/// refused because nothing above level 90 exists in the game.
///
/// `u64` and `i64` do not satisfy `f64: From<T>`, since the standard library provides no lossy
/// `From`. Cast with `as f64` first.
///
/// # Examples
///
/// ```
/// use core::utils::lerp;
///
/// assert_eq!(lerp(0u32, 89u32, 1, 90), Some(0.0));
/// assert_eq!(lerp(0u32, 89u32, 90, 90), Some(89.0));
/// assert_eq!(lerp(0u32, 89u32, 46, 90), Some(45.0));
///
/// assert_eq!(lerp(0u32, 89u32, 0, 90), None);
/// assert_eq!(lerp(0u32, 89u32, 91, 90), None);
/// ```
pub fn lerp<T>(start: T, end: T, lvl: usize, lvl_max: usize) -> Option<f64>
where
    f64: From<T>,
{
    if lvl == 0 || lvl_max <= 1 || lvl > lvl_max {
        return None;
    }
    let start_f64: f64 = start.into();
    let end_f64: f64 = end.into();

    Some(start_f64 + (end_f64 - start_f64) * (lvl - 1) as f64 / (lvl_max - 1) as f64)
}
