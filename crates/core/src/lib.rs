//! # core
//!
//! The pieces every other crate builds on.
//!
//! `damage` is the notable one: damage is keyed by which student skills are active, so looking
//! it up is O(1).

pub mod actions;
pub mod agent;
pub mod algorithm;
pub mod base;
pub mod boss;
pub mod character;
pub mod constants;
pub mod damage;
pub mod difficulty;
pub mod effect;
pub mod extra;
pub mod locale;
pub mod simulator;
pub mod skill;
pub mod stat;
pub mod state;
pub mod student;
pub mod table;
pub mod terrains;
pub mod types;
pub mod uid;
pub mod utils;

pub use std::default;
pub use std::marker;
