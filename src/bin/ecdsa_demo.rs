#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import crypto utilities
use stm32_tests::utils::crypto::{bytes_to_hex_string, keccak256, EthereumKeyPair, SimpleRng};

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 2000) {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== STM32 Ethereum ECDSA Signing Demo ===");
    rprintln!("Using secp256k1 curve (Ethereum standard)");
    rprintln!("Press button to generate keys and sign message");

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

    // Enable DWT cycle counter for precise timing
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    // Turn LED off initially (LED is active low on STM32 Blackpill)
    led.set_high();

    rprintln!("Ready! Press button to start ECDSA demo...");

    loop {
        // Wait for button press
        while button.is_high() {
            // Wait for button to be pressed
        }

        rprintln!("\n=== Starting ECDSA Demo ===");

        // Get start time (using DWT cycle counter for precise timing)
        let start_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("Start cycles: {}", start_cycles);

        // Step 1: Generate Ethereum key pair
        rprintln!("\n[1/5] Generating Ethereum key pair (secp256k1)...");
        // Use a combination of cycle counter and a changing value for seed
        let seed = start_cycles.wrapping_add(cortex_m::peripheral::DWT::cycle_count());
        rprintln!("RNG seed: 0x{:08X}", seed);

        let mut rng = SimpleRng::new(seed);
        let keypair = EthereumKeyPair::generate(&mut rng);

        rprintln!("✓ Ethereum key pair generated successfully");

        // Print private key (32 bytes = 64 hex chars)
        let private_key_bytes = keypair.private_key_bytes();
        let mut priv_hex_buffer = [0u8; 64];
        let priv_hex = bytes_to_hex_string(&private_key_bytes, &mut priv_hex_buffer);
        rprintln!("  Private Key: 0x{}", priv_hex);

        // Print public key (compressed - 33 bytes = 66 hex chars)
        let pub_bytes = keypair.public_key_compressed_bytes();
        let mut pub_hex_buffer = [0u8; 66];
        let pub_hex = bytes_to_hex_string(&pub_bytes, &mut pub_hex_buffer);
        rprintln!("  Public Key (compressed): 0x{}", pub_hex);

        // Print Ethereum address (20 bytes = 40 hex chars)
        let mut addr_hex_buffer = [0u8; 40];
        let addr_hex = bytes_to_hex_string(&keypair.address, &mut addr_hex_buffer);
        rprintln!("  Ethereum Address: 0x{}", addr_hex);

        let step1_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step1_cycles.wrapping_sub(start_cycles));

        // Step 2: Create message and hash it
        rprintln!("\n[2/5] Creating message to sign...");
        let message = b"Hello, Ethereum ECDSA on STM32!";
        rprintln!("  Message: {:?}", core::str::from_utf8(message).unwrap());

        // Hash the message using Keccak256 (Ethereum standard)
        let message_hash = keccak256(message);
        rprintln!(
            "  Keccak256 hash: [{:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}]...",
            message_hash[0],
            message_hash[1],
            message_hash[2],
            message_hash[3],
            message_hash[4],
            message_hash[5],
            message_hash[6],
            message_hash[7]
        );

        let step2_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step2_cycles.wrapping_sub(step1_cycles));

        // Step 3: Sign the message
        rprintln!("\n[3/5] Signing message with ECDSA...");
        let signature = keypair.sign(message);
        rprintln!("✓ Signature generated successfully");
        let sig_bytes = signature.to_bytes();
        rprintln!("  Signature (r,s): [{:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}]...",
                  sig_bytes[0], sig_bytes[1], sig_bytes[2], sig_bytes[3],
                  sig_bytes[4], sig_bytes[5], sig_bytes[6], sig_bytes[7]);

        let step3_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step3_cycles.wrapping_sub(step2_cycles));

        // Step 4: Blink LED to show signature was generated
        rprintln!("\n[4/5] Blinking LED (signature generated)...");
        led.set_low(); // LED on
        delay_ms(200);
        led.set_high(); // LED off
        delay_ms(200);
        rprintln!("✓ LED blinked once");

        let step4_cycles = cortex_m::peripheral::DWT::cycle_count();

        // Step 5: Verify the signature
        rprintln!("\n[5/5] Verifying signature...");
        match keypair.verify(message, &signature) {
            Ok(_) => {
                rprintln!("✓ Signature verification SUCCESSFUL!");

                let step5_cycles = cortex_m::peripheral::DWT::cycle_count();
                rprintln!("  Cycles: {}", step5_cycles.wrapping_sub(step4_cycles));

                // Blink LED 3 times to show verification complete
                rprintln!("\nBlinking LED 3 times (verification complete)...");
                for _i in 1..=3 {
                    rprintln!("  Blink {}/3", _i);
                    led.set_low(); // LED on
                    delay_ms(150);
                    led.set_high(); // LED off
                    delay_ms(150);
                }

                // Calculate total time
                let end_cycles = cortex_m::peripheral::DWT::cycle_count();
                let total_cycles = end_cycles.wrapping_sub(start_cycles);
                rprintln!("\n=== Demo Complete ===");
                rprintln!("Total cycles: {}", total_cycles);
                rprintln!("Approximate time: ~{} ms", total_cycles / 84000); // 84 MHz clock
            }
            Err(_) => {
                rprintln!("✗ Signature verification FAILED!");
                // Blink LED rapidly to indicate error
                for _ in 0..5 {
                    led.set_low();
                    delay_ms(50);
                    led.set_high();
                    delay_ms(50);
                }
            }
        }

        rprintln!("\nPress button to run demo again...");

        // Wait for button release before next iteration
        while button.is_low() {
            // Wait for button to be released
        }

        delay_ms(200); // Debounce delay
    }
}
