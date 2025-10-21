#![no_std]

//! STM32 Utilities Library
//!
//! This library provides reusable utilities for STM32 embedded projects,
//! including display drivers, cryptographic functions, and other common
//! functionality that can be shared across multiple binaries.

extern crate alloc;

use alloc_cortex_m::CortexMHeap;

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub mod utils;

// Re-export commonly used items at the crate root for convenience
pub use utils::oled::{DcPin, OledDisplay, RstPin};

// Custom getrandom implementation for no_std embedded targets
use getrandom::register_custom_getrandom;

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Use DWT cycle counter as entropy source
    // This is not cryptographically secure but works for embedded demos
    use cortex_m::peripheral::DWT;

    for chunk in buf.chunks_mut(4) {
        let random = DWT::cycle_count();
        let bytes = random.to_le_bytes();
        for (i, byte) in chunk.iter_mut().enumerate() {
            *byte = bytes[i];
        }
    }
    Ok(())
}

register_custom_getrandom!(custom_getrandom);
