#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import crypto utilities and RngCore trait
use rand_core::RngCore;
use stm32_tests::utils::crypto::{keccak256, SimpleRng};

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 2000) {
        cortex_m::asm::nop();
    }
}

/// Check if hash meets the difficulty target (leading zeros with threshold)
/// Uses a two-part difficulty: full_bytes + threshold for next byte
fn check_difficulty(hash: &[u8; 32], full_zero_bytes: u8, threshold: u8) -> bool {
    // First check that the required number of bytes are fully zero
    for i in 0..(full_zero_bytes as usize) {
        if i >= hash.len() {
            return true;
        }
        if hash[i] != 0 {
            return false;
        }
    }

    // Then check if the next byte is below the threshold
    let next_byte_idx = full_zero_bytes as usize;
    if next_byte_idx < hash.len() && threshold < 255 {
        hash[next_byte_idx] < threshold
    } else {
        true
    }
}

/// Format hash as hex string for display
fn format_hash_short<'a>(hash: &[u8; 32], buffer: &'a mut [u8; 16]) -> &'a str {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    // Show first 8 bytes (16 hex chars)
    for i in 0..8 {
        buffer[i * 2] = HEX_CHARS[(hash[i] >> 4) as usize];
        buffer[i * 2 + 1] = HEX_CHARS[(hash[i] & 0x0F) as usize];
    }
    core::str::from_utf8(buffer).unwrap()
}

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== STM32 Hashcash Proof-of-Work Demo ===");
    rprintln!("Press button to start mining!");

    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Get RCC peripheral for enabling GPIO clocks
    let mut rcc = dp.RCC;

    // Setup GPIOC (PC13 LED)
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();

    // Setup GPIOA (PA0 Button)
    let gpioa = dp.GPIOA.split(&mut rcc);
    let button = gpioa.pa0.into_pull_up_input();

    // Enable DWT cycle counter for timing and entropy
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    // Turn LED off initially (LED is active low on STM32 Blackpill)
    led.set_high();

    // Startup blink sequence
    for _ in 0..3 {
        led.set_low();
        delay_ms(100);
        led.set_high();
        delay_ms(100);
    }

    rprintln!("\nReady! Press button to start continuous mining...");
    rprintln!("Each solution will chain into the next block!");

    // Difficulty setting: (full_zero_bytes, threshold_for_next_byte)
    // (0, 16) = easy (~6% success rate, ~16 attempts)
    // (0, 4) = medium (~1.5% success rate, ~64 attempts)
    // (1, 0) = harder (~0.39% success rate, ~256 attempts)
    // (1, 16) = between 1 & 2 (~0.024% success rate, ~4k attempts)
    // (1, 4) = closer to 2 (~0.006% success rate, ~16k attempts)
    // (2, 0) = hard (~0.0015% success rate, ~65k attempts)
    const FULL_ZERO_BYTES: u8 = 1;
    const THRESHOLD: u8 = 16; // Next byte must be < this value

    // Wait for button press to start
    while button.is_high() {
        // Wait for button to be pressed
    }

    rprintln!("\n=== BUTTON PRESSED - STARTING CONTINUOUS MINING ===");

    // Generate initial random input data
    let start_cycles = cortex_m::peripheral::DWT::cycle_count();
    let seed = start_cycles.wrapping_add(cortex_m::peripheral::DWT::cycle_count());
    let mut rng = SimpleRng::new(seed);

    let mut input_data = [0u8; 32];
    for i in 0..8 {
        let random_u32 = RngCore::next_u32(&mut rng);
        let bytes = random_u32.to_le_bytes();
        input_data[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }

    let mut hex_buffer = [0u8; 16];
    let input_hex = format_hash_short(&input_data, &mut hex_buffer);
    rprintln!("\nInitial input: {}...", input_hex);
    rprintln!(
        "Difficulty: {} zero byte(s) + next byte < 0x{:02x}\n",
        FULL_ZERO_BYTES,
        THRESHOLD
    );

    let mut block_number = 0u32;
    let mut total_attempts = 0u64;

    // Continuous mining loop
    loop {
        block_number += 1;
        let block_start_cycles = cortex_m::peripheral::DWT::cycle_count();

        rprintln!("=== Block #{} ===", block_number);
        let input_hex = format_hash_short(&input_data, &mut hex_buffer);
        rprintln!("  Input: {}...", input_hex);
        rprintln!("  Mining...");

        led.set_low(); // Turn LED on during mining

        let mut nonce: u32 = 0;
        let mut hash: [u8; 32];
        let mut attempts = 0u32;

        loop {
            // Prepare data: input_data || nonce (32 + 4 = 36 bytes)
            let mut data = [0u8; 36];
            data[0..32].copy_from_slice(&input_data);
            data[32..36].copy_from_slice(&nonce.to_le_bytes());

            // Hash the data
            hash = keccak256(&data);

            attempts += 1;

            // Check if we found a valid hash
            if check_difficulty(&hash, FULL_ZERO_BYTES, THRESHOLD) {
                break;
            }

            // Increment nonce and try again
            nonce = nonce.wrapping_add(1);

            // Print progress every 1000 attempts
            if attempts % 1000 == 0 {
                rprintln!("  Attempts: {} (nonce: {})", attempts, nonce);
            }
        }

        led.set_high(); // Turn LED off after mining

        let mining_cycles =
            cortex_m::peripheral::DWT::cycle_count().wrapping_sub(block_start_cycles);
        total_attempts += attempts as u64;

        // Victory blink sequence - fast blinks (10 times)
        for _ in 0..10 {
            led.set_low();
            delay_ms(50);
            led.set_high();
            delay_ms(50);
        }

        rprintln!("  ✓ Found! Nonce: {} (attempts: {})", nonce, attempts);
        let hash_hex = format_hash_short(&hash, &mut hex_buffer);
        rprintln!("  Hash: {}...", hash_hex);
        rprintln!("  Time: ~{} ms", mining_cycles / 84000);
        rprintln!("  Total attempts: {}", total_attempts);
        rprintln!(
            "  Avg hash rate: ~{} H/s\n",
            (total_attempts * 84000000)
                / cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start_cycles) as u64
        );

        // Use the hash as input for the next block (blockchain!)
        input_data = hash;
    }
}
