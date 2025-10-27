//! Custom getrandom implementation for STM32 no-std environment
//!
//! The actual implementation is in src/lib.rs as `getrandom_custom()`
//! which is called by getrandom when configured with getrandom_backend="custom"
//!
//! WARNING: The current implementation uses DWT cycle counter which is NOT
//! cryptographically secure. For production use, you should:
//! 1. Use your MCU's hardware RNG if available
//! 2. Use an external entropy source (ADC noise, temperature sensor, etc.)
//! 3. Consider using rand_chacha with a seed from a secure source

// This module exists for documentation purposes
// The actual getrandom implementation is in src/lib.rs

// TODO: For production, implement a secure RNG using STM32 hardware RNG:
//
// Example using STM32 hardware RNG (pseudocode):
// ```
// use stm32f4xx_hal::rng::Rng;
//
// static mut RNG_INSTANCE: Option<Rng> = None;
//
// pub fn init_rng(rng: Rng) {
//     unsafe {
//         RNG_INSTANCE = Some(rng);
//     }
// }
//
// #[no_mangle]
// pub fn getrandom_custom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
//     unsafe {
//         if let Some(rng) = &mut RNG_INSTANCE {
//             for byte in buf.iter_mut() {
//                 *byte = rng.gen::<u8>();
//             }
//             Ok(())
//         } else {
//             Err(getrandom::Error::UNSUPPORTED)
//         }
//     }
// }
