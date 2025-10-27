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
// This is configured via the getrandom_backend="custom" cfg flag in .cargo/config.toml
#[no_mangle]
pub extern "C" fn __getrandom_v03_custom(buf: *mut u8, len: usize) -> i32 {
    // Use DWT cycle counter as entropy source
    // This is not cryptographically secure but works for embedded demos
    use cortex_m::peripheral::DWT;

    let buf_slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };

    for chunk in buf_slice.chunks_mut(4) {
        let random = DWT::cycle_count();
        let bytes = random.to_le_bytes();
        for (i, byte) in chunk.iter_mut().enumerate() {
            *byte = bytes[i];
        }
    }
    0 // Success
}
