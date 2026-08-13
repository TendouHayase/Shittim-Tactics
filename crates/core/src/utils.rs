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
            let sign_bit = cross_product(valid_region[0], valid_region[1], valid_region[0], p)
                .0
                .to_bits()
                & 0x80;
            let mut is_include = 0;

            for idx in 1..count {
                let s = cross_product(
                    valid_region[idx as usize],
                    valid_region[idx as usize % count as usize],
                    valid_region[idx as usize],
                    p,
                )
                .0
                .to_bits()
                    & 0x80;

                // 기존 부호와 같은지 비교
                is_include = sign_bit ^ s;
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
            let radian = cross.atan2(*dot);

            start_angle_degree as f32 <= radian.to_degrees()
                && radian.to_degrees() <= end_angle_degree as f32
        }
    }
}

#[macro_export]
macro_rules! count_token_trees {
    () => {
        0usize
    };

    ($head:tt $($tail:tt)*) => (1usize + count_token_trees!($($tail)*))
}

#[macro_export]
macro_rules! count_types {
    ($($ty:ty),* $(,)?) => {
        count_token_trees!($({$ty})*)
    };
}

#[macro_export]
macro_rules! tuple_for {
    ($var:ident in $tuple:expr; [$($idx:tt),+ $(,)?] => $body:block) => {
        $(
            {
                let $var = &$tuple.$idx;
                $body
            }
        )+
    };
}

#[macro_export]
macro_rules! tuple_for_move {
    ($var:ident in $tuple:expr; [$($idx:tt),+ $(,)?] => $body:block) => {
        $(
            {
                let $var = $tuple.$idx;
                $body
            }
        )+
    };
}

#[macro_export]
macro_rules! variant_accessor {
    // Unit variant
    ($enum:ty, $variant:ident, $name:ident) => {
        paste::paste! {
            impl $enum {
                #[inline]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self, $enum::$variant)
                }
            }
        }
    };

    // 단일 필드 tuple variant: 타입만 명시
    ($enum:ty, $variant:ident($ty:ty), $name:ident) => {
        paste::paste! {
            impl $enum {
                #[inline]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self, $enum::$variant(_))
                }

                #[inline]
                pub fn [<as_ $name>](&self) -> Option<&$ty> {
                    match self {
                        $enum::$variant(v) => Some(v),
                        _ => None,
                    }
                }
            }
        }
    };

    // 다중 필드 tuple variant: 바인더 이름을 함께 명시
    ($enum:ty, $variant:ident($($field:ident : $ty:ty),+ $(,)?), $name:ident) => {
        paste::paste! {
            impl $enum {
                #[inline]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self, $enum::$variant(..))
                }

                #[inline]
                pub fn [<as_ $name>](&self) -> Option<($(&$ty),+)> {
                    match self {
                        $enum::$variant($($field),+) => Some(($($field),+)),
                        _ => None,
                    }
                }

                $(
                    #[inline]
                    pub fn [<as_ $name _ $field>](&self) -> Option<&$ty> {
                        match self {
                            $enum::$variant($($field),+) => Some($field),
                            _ => None,
                        }
                    }
                )+
            }
        }
    };

    // named struct variant
    ($enum:ty, $variant:ident { $($field:ident : $ty:ty),+ $(,)?}, $name:ident) => {
        paste::paste! {
            impl $enum {
                #[inline]
                pub fn [<is_ $name>](&self) -> bool {
                    matches!(self, $enum::$variant { .. })
                }

                #[inline]
                pub fn [<as_ $name>](&self) -> Option<($(&$ty),+)> {
                    match self {
                        $enum::$variant { $($field),+ } => Some(($($field),+)),
                        _ => None,
                    }
                }

                $(
                    #[inline]
                    pub fn [<as_ $name _ $field>](&self) -> Option<&$ty> {
                        match self {
                            $enum::$variant { $field, .. } => Some($field),
                            _ => None,
                        }
                    }
                )+
            }
        }
    };
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

/// Ordinary least squares fit of `y = mx + b`, returning `(m, b)`.
///
/// # Panics
///
/// When `x` and `y` differ in length.
pub fn ols<T: Clone>(x: &[T], y: &[T]) -> (f64, f64)
where
    f64: From<T>,
{
    assert!(x.len() == y.len(), "ols: x and y must have equal length");
    let mut m_num: f64 = 0.0;
    let mut m_den: f64 = 0.0;

    let x_bar: f64 = x.iter().map(|x| f64::from(x.clone())).sum::<f64>() / x.len() as f64;
    let y_bar: f64 = y.iter().map(|x| f64::from(x.clone())).sum::<f64>() / y.len() as f64;

    for i in 0..x.len() {
        m_num += (f64::from(x[i].clone()) - x_bar) * (f64::from(y[i].clone()) - y_bar);
        m_den += (f64::from(x[i].clone()) - x_bar).powi(2);
    }

    let m = m_num / m_den;

    (m, y_bar - m * x_bar)
}

/// Intersection of closed intervals.
///
/// `None` when the iterator is empty, when the intersection is empty, or when any bound is NaN
/// or infinite. The last case is checked explicitly: `f64::max` and `f64::min` ignore NaN, so a
/// NaN bound would otherwise drop out silently and read as an unconstrained interval.
///
/// # Examples
///
/// ```
/// use core::utils::intersect_intervals;
///
/// let overlap = intersect_intervals([(0.0, 3.0), (1.0, 5.0)].into_iter());
/// assert_eq!(overlap, Some((1.0, 3.0)));
///
/// let disjoint = intersect_intervals([(0.0, 1.0), (2.0, 3.0)].into_iter());
/// assert_eq!(disjoint, None);
/// ```
pub fn intersect_intervals(it: impl Iterator<Item = (f64, f64)>) -> Option<(f64, f64)> {
    let mut it = it.peekable();
    if it.peek().is_none() {
        return None;
    }

    let mut left = f64::MIN;
    let mut right = f64::MAX;

    for (lhs, rhs) in it {
        if !lhs.is_finite() || !rhs.is_finite() {
            return None;
        }
        left = left.max(lhs);
        right = right.min(rhs);
    }

    if left > right {
        None
    } else {
        Some((left, right))
    }
}

/// `(m, b)` of the line `y = mx + b` through two points.
///
/// A vertical pair yields an infinite or NaN slope, returned as is.
///
/// # Examples
///
/// ```
/// use core::utils::line_through;
///
/// assert_eq!(line_through((1.0, 3.0), (3.0, 7.0)), (2.0, 1.0));
/// ```
pub fn line_through(p1: (f64, f64), p2: (f64, f64)) -> (f64, f64) {
    let m = (p2.1 - p1.1) / (p2.0 - p1.0);
    let b = p1.1 - m * p1.0;

    (m, b)
}
