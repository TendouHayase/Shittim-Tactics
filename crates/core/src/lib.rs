//! # core
//!
//! 타 크레이트를 구현하는데 필요한 핵심 요소들을 모아둔 크레이트입니다.
//!
//! ## 주요 모듈
//! - 'damage' : 데미지 관련
//! * 데미지를 학생들 스킬 발동 여부로 O(1)에 구하도록 구현

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
pub mod utils;

pub use std::default;
pub use std::marker;
