//! Utility modules for STM32 projects
//!
//! This module contains reusable utilities that can be shared across
//! multiple binaries, including display drivers, crypto implementations,
//! and other common functionality.

pub mod crypto;
pub mod groth16;
pub mod oled;

// Re-export commonly used types for convenience
pub use oled::{DcPin, OledDisplay, RstPin};
