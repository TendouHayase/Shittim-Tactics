use ordered_float::OrderedFloat;

use crate::skill::Region;

pub const TPS: u16 = 30;
pub const MAX_STUDENT_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Default, Eq, Hash)]
pub struct Position {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
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

macro_rules! count_token_trees {
    () => {
        0usize
    };

    ($head:tt $($tail:tt)*) => (1usize + count_token_trees!($($tail)*))
}

macro_rules! count_types {
    ($($ty:ty),* $(,)?) => {
        count_token_trees!($({$ty})*)
    };
}
