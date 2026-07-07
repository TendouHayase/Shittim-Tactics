pub mod actions;
pub mod agent;
pub mod base;
pub mod boss;
pub mod character;
pub mod damage;
pub mod difficulty;
pub mod simulator;
pub mod skill;
pub mod state;
pub mod student;
pub mod terrains;
pub mod types;

const TPS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub use std::default;
pub use std::marker;

use ordered_float::OrderedFloat;
