use crate::skill::Region;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const TPS: u16 = 30;
pub const MAX_STUDENT_COUNT: usize = 10;

/// json에 소수로 적힌 값을 반올림 없이 담는 유리수. 분모는 항상 `10^exp`.
///
/// 부동소수로 접으면 스탯 몇십 차이가 반올림에서 갈리고 그게 택틱 성패를 바꾸므로, 곱셈을
/// 먼저 하고 나눗셈을 마지막에 하려고 분자와 자릿수를 따로 들고 있음.
///
/// # Warning
///
/// json 숫자는 serde가 이미 `f64`로 만들어 넘겨주기 때문에 원문 문자열을 볼 수 없음. 대신
/// `f64`의 최단 왕복 표현(`{}` 포맷)에서 자릿수를 다시 읽어냄. `26.8`처럼 소수 몇 자리짜리
/// 게임 데이터는 이 왕복으로 정확히 복원되지만, 유효숫자 17자리를 넘는 값을 적으면 복원되지
/// 않음.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Ratio {
    num: i64,
    exp: u8,
}

impl Ratio {
    /// `10^exp`가 `i64`를 넘지 않는 한계.
    pub const MAX_EXP: u8 = 18;

    /// 뒤따르는 0을 떼어 같은 값이 항상 같은 표현을 갖게 함. `Eq`/`Hash`가 값 비교가 되려면
    /// 필요함 (`2.50`과 `2.5`).
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

    /// `value * self`를 정수로. 중간 반올림이 없도록 곱셈이 먼저 가고, 나눗셈은 0 쪽으로 버림.
    /// 곱한 값이 `i64`를 넘길 수 있어 `i128`을 거침.
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

#[derive(Debug, Clone, Copy, PartialEq, Default, Eq, Hash)]
pub struct Position {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

/// json에서는 `[x, y]`. `ordered-float`의 serde 기능을 켜지 않으려고 직접 붙인 것이고, 덤으로
/// 데이터 파일에 필드 이름이 반복되지 않음.
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

use std::ops::Add;
use std::ops::Sub;

#[inline]
pub fn euclidean_distance(lhs: Position, rhs: Position) -> f64 {
    ((lhs.x - rhs.x) * (lhs.x - rhs.x) + (lhs.y - rhs.y) * (lhs.y - rhs.y))
        .sqrt()
        .into()
}

#[inline]
/// p 벡터와 q벡터 크로스곱
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
