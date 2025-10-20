#![no_std]

//! STM32 Utilities Library
//!
//! This library provides reusable utilities for STM32 embedded projects,
//! including display drivers, cryptographic functions, and other common
//! functionality that can be shared across multiple binaries.

pub mod utils;

// Re-export commonly used items at the crate root for convenience
pub use utils::oled::{DcPin, OledDisplay, RstPin};
