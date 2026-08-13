//! # core
//!
//! The pieces every other crate builds on.
//!
//! `damage` is the notable one: damage is keyed by which student skills are active, so looking
//! it up is O(1).

pub mod actions;
pub mod agent;
pub mod base;
pub mod boss;
pub mod character;
pub mod constants;
pub mod damage;
pub mod difficulty;
pub mod effect;
pub mod locale;
pub mod simulator;
pub mod skill;
pub mod stat;
pub mod state;
pub mod student;
pub mod table;
pub mod terrains;
pub mod types;
pub mod utils;

pub use std::default;
pub use std::marker;

// 아래 세 모듈은 `cargo xtask`가 만듦. 저장소에 없으므로 클론 직후에는 파일이 없고, 그래서
// 생성기를 안 돌리면 여기서 컴파일이 멈춤. 선언 자체는 xtask가 넣으므로 지워도 다시 생김.
pub mod boss_macros;
pub mod skills;
pub mod states;
