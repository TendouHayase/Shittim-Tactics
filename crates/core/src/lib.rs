pub mod actions;
pub mod agent;
pub mod base;
pub mod boss;
pub mod character;
pub mod damage;
pub mod simulator;
pub mod skill;
pub mod state;
pub mod student;
pub mod terrains;
pub mod types;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    x: OrderedFloat<f32>,
    y: OrderedFloat<f32>,
}

pub use std::default;
pub use std::marker;

use ordered_float::OrderedFloat;
