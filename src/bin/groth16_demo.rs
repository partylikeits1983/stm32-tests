#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Enable alloc for arkworks
extern crate alloc;
use core::alloc::Layout;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import Groth16 utilities
use stm32_tests::utils::groth16::{self, vk_proof};

// Global allocator for arkworks
use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 2000) {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    // Initialize the allocator with 64KB heap
    // Adjust size based on your STM32's RAM
    const HEAP_SIZE: usize = 64 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        let heap_ptr = core::ptr::addr_of!(HEAP) as *const u8;
        ALLOCATOR.init(heap_ptr as usize, HEAP_SIZE)
    }

    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== STM32 Groth16 Proof Verification Demo ===");
    rprintln!("Verifying IMT preimage proof on BN254 curve");

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

    // Load and prepare verification key once (expensive operation)
    rprintln!("\nPreparing verification key...");
    let vk_data = vk_proof::verification_key();

    // Convert to groth16::Vk
    let vk = groth16::Vk {
        alpha_g1: vk_data.alpha_g1,
        beta_g2: vk_data.beta_g2,
        gamma_g2: vk_data.gamma_g2,
        delta_g2: vk_data.delta_g2,
        ic: &vk_data.ic,
    };

    let pvk = vk.prepare();
    rprintln!("✓ Verification key prepared");

    rprintln!("Ready! Press button to start verification...");

    loop {
        // Wait for button press
        while button.is_high() {
            // Wait for button to be pressed
        }

        rprintln!("\n=== Starting Groth16 Verification ===");

        // Get start time
        let start_cycles = cortex_m::peripheral::DWT::cycle_count();

        // Step 1: Turn LED on to indicate verification started
        led.set_low(); // LED on
        rprintln!("[1/3] Loading proof and public inputs...");

        // Load the sample proof and public inputs
        let (proof_data, public_inputs) = vk_proof::sample_proof();

        // Convert to groth16::Proof
        let proof = groth16::Proof {
            a: proof_data.a,
            b: proof_data.b,
            c: proof_data.c,
        };

        rprintln!("  Proof loaded:");
        rprintln!("    - Proof.a (G1 point)");
        rprintln!("    - Proof.b (G2 point)");
        rprintln!("    - Proof.c (G1 point)");
        rprintln!("  Public inputs: {} field elements", public_inputs.len());

        let step1_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step1_cycles.wrapping_sub(start_cycles));

        // Step 2: Verify the proof
        rprintln!("\n[2/3] Verifying Groth16 proof...");
        rprintln!("  This involves pairing checks on BN254 curve");
        rprintln!("  Please wait (this may take several seconds)...");

        let verify_start = cortex_m::peripheral::DWT::cycle_count();

        let result = groth16::verify_proof_prepared(&pvk, &proof, &public_inputs);

        let verify_end = cortex_m::peripheral::DWT::cycle_count();
        let verify_cycles = verify_end.wrapping_sub(verify_start);

        match result {
            Ok(_) => {
                rprintln!("✓ Proof verification SUCCESSFUL!");
                rprintln!("  Verification cycles: {}", verify_cycles);
                rprintln!("  Approximate time: ~{} ms", verify_cycles / 84000); // 84 MHz clock

                // Step 3: Blink LED rapidly to indicate success
                rprintln!("\n[3/3] Verification successful...");
                for _ in 1..=6 {
                    led.set_high(); // LED off
                    delay_ms(120);
                    led.set_low(); // LED on
                    delay_ms(120);
                }
                led.set_high(); // LED off at end
            }
            Err(_) => {
                rprintln!("✗ Proof verification FAILED!");
                rprintln!("  Verification cycles: {}", verify_cycles);

                // Step 3: Blink LED slowly to indicate failure
                rprintln!("\n[3/3] Blinking LED (verification failed)...");
                for _ in 1..=20 {
                    led.set_high(); // LED off
                    delay_ms(400);
                    led.set_low(); // LED on
                    delay_ms(400);
                }
                led.set_high(); // LED off at end
            }
        }

        // Calculate total time
        let end_cycles = cortex_m::peripheral::DWT::cycle_count();
        let total_cycles = end_cycles.wrapping_sub(start_cycles);
        rprintln!("\n=== Verification Complete ===");
        rprintln!("Total cycles: {}", total_cycles);
        rprintln!("Approximate total time: ~{} ms", total_cycles / 84000);

        rprintln!("\nPress button to verify again...");

        // Wait for button release before next iteration
        while button.is_low() {
            // Wait for button to be released
        }

        delay_ms(200); // Debounce delay
    }
}

// Required for alloc
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    rprintln!("ALLOC ERROR!");
    loop {
        cortex_m::asm::nop();
    }
}
