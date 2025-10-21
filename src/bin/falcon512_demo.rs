#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import Falcon512 utilities
use stm32_tests::utils::crypto::SimpleRng;
use stm32_tests::utils::falcon::Falcon512KeyPair;

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
    rprintln!("=== STM32 Falcon512 Post-Quantum Signing Demo ===");
    rprintln!("Using Falcon512 (NIST Level 1 security)");
    rprintln!("Press button to generate keys and sign message");

    // Initialize heap for miden-crypto allocations
    // Reduced to fit in STM32F411's 128KB RAM
    const HEAP_SIZE: usize = 96 * 1024; // 96KB heap (leaves ~32KB for stack and other data)
    static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        stm32_tests::ALLOCATOR.init(
            core::ptr::addr_of!(HEAP_MEM) as *const _ as usize,
            HEAP_SIZE,
        )
    }

    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Get RCC peripheral for enabling GPIO clocks
    let mut rcc = dp.RCC;

    // Setup GPIOC (PC13 LED)
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();

    for _ in 0..5 {
        led.set_low();
        delay_ms(100);
        led.set_high();
        delay_ms(100);
    }

    // Setup GPIOA (PA0 Button)
    let gpioa = dp.GPIOA.split(&mut rcc);
    let button = gpioa.pa0.into_pull_up_input();

    // Enable DWT cycle counter for precise timing
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    // Turn LED off initially (LED is active low on STM32 Blackpill)
    led.set_high();

    rprintln!("Ready! Press button to start Falcon512 demo...");

    loop {
        // Wait for button press
        while button.is_high() {
            // Wait for button to be pressed
        }

        // IMMEDIATE feedback to confirm button press and RTT working
        rprintln!("\n=== BUTTON PRESSED ===");
        rprintln!("Starting Falcon512 Demo...");

        // Get start time (using DWT cycle counter for precise timing)
        let start_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("Start cycles: {}", start_cycles);

        // Step 1: Generate Falcon512 key pair
        rprintln!("\n[1/6] Generating Falcon512 key pair...");
        rprintln!("WARNING: This may take 1-5 minutes on STM32F411 @ 72MHz");
        rprintln!("LED will blink twice to show we're starting...");

        // Blink LED to show we're starting key generation
        for _ in 0..10 {
            led.set_low();
            delay_ms(200);
            led.set_high();
            delay_ms(200);
        }

        rprintln!("Starting key generation NOW...");

        // Create simple RNG using DWT cycle counter
        let seed_value = start_cycles.wrapping_add(cortex_m::peripheral::DWT::cycle_count());
        rprintln!("RNG seed: 0x{:08X}", seed_value);

        let mut rng = SimpleRng::new(seed_value);
        let keypair = Falcon512KeyPair::generate(&mut rng);

        rprintln!("✓ Falcon512 key pair generated successfully");

        // Print public key as Word (4 field elements)
        let pub_word = keypair.public_key_word();
        rprintln!(
            "  Public Key (as Word): [{:?}, {:?}, {:?}, {:?}]",
            pub_word[0],
            pub_word[1],
            pub_word[2],
            pub_word[3]
        );

        let step1_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step1_cycles.wrapping_sub(start_cycles));

        // Step 2: Blink LED when ready
        rprintln!("\n[2/6] Ready - blinking LED...");
        for _ in 0..50 {
            led.set_low();
            delay_ms(200);
            led.set_high();
            delay_ms(200);
        }

        let step2_cycles = cortex_m::peripheral::DWT::cycle_count();

        // Step 3: Create message to sign
        rprintln!("\n[3/6] Creating message to sign...");
        let message = b"Hello, Falcon512 Post-Quantum Crypto on STM32!";
        rprintln!("  Message: {:?}", core::str::from_utf8(message).unwrap());
        rprintln!("  Message length: {} bytes", message.len());

        let step3_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step3_cycles.wrapping_sub(step2_cycles));

        // Step 4: Sign the message (LED on during signing)
        rprintln!("\n[4/6] Signing message with Falcon512...");
        led.set_low(); // LED on during signing

        let signature = keypair.sign(message, &mut rng);

        led.set_high(); // LED off after signing
        rprintln!("✓ Signature generated successfully");
        rprintln!("  Signature generated (Falcon512 format)");

        let step4_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step4_cycles.wrapping_sub(step3_cycles));

        // Step 5: Blink LED to show signing complete
        rprintln!("\n[5/6] Signing complete - blinking LED...");
        for _ in 0..50 {
            led.set_low();
            delay_ms(300);
            led.set_high();
            delay_ms(300);
        }

        let step5_cycles = cortex_m::peripheral::DWT::cycle_count();

        // Step 6: Verify the signature (LED on during verification)
        rprintln!("\n[6/6] Verifying signature...");
        led.set_low(); // LED on during verification

        let is_valid = keypair.verify(message, &signature);

        if is_valid {
            rprintln!("✓ Signature verification SUCCESSFUL!");

            let step6_cycles = cortex_m::peripheral::DWT::cycle_count();
            rprintln!("  Cycles: {}", step6_cycles.wrapping_sub(step5_cycles));

            // Keep LED ON to indicate successful signature verification
            rprintln!("\nKeeping LED ON (signature verified successfully)...");
            led.set_low(); // LED on (active low)

            // Calculate total time
            let end_cycles = cortex_m::peripheral::DWT::cycle_count();
            let total_cycles = end_cycles.wrapping_sub(start_cycles);
            rprintln!("\n=== Demo Complete ===");
            rprintln!("Total cycles: {}", total_cycles);
            rprintln!("Approximate time: ~{} ms", total_cycles / 84000); // 84 MHz clock

            rprintln!("\n=== Falcon512 Security Info ===");
            rprintln!("Security Level: NIST Level 1 (128-bit quantum security)");
            rprintln!("Public Key: Word (4 field elements)");
            rprintln!("Signature: Falcon512 format");
            rprintln!("Post-Quantum: Resistant to quantum computer attacks");
            rprintln!("Algorithm: Lattice-based (NTRU lattices)");
        } else {
            rprintln!("✗ Signature verification FAILED!");
            // Blink LED rapidly to indicate error
            for _ in 0..20 {
                led.set_low();
                delay_ms(50);
                led.set_high();
                delay_ms(50);
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
