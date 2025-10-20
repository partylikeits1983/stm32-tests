#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// ECDSA and crypto imports for Ethereum (secp256k1)
use k256::{
    ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey},
    elliptic_curve::{
        rand_core::{CryptoRng, RngCore},
        sec1::ToEncodedPoint,
    },
    PublicKey,
};
use sha3::{Digest, Keccak256};

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 2000) {
        cortex_m::asm::nop();
    }
}

// Simple RNG implementation using hardware timer as entropy source
struct SimpleRng {
    state: u32,
    counter: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        SimpleRng {
            state: seed,
            counter: 0,
        }
    }

    fn next_u32_internal(&mut self) -> u32 {
        // Simple LCG (Linear Congruential Generator) with counter mixing
        self.counter = self.counter.wrapping_add(1);
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state ^ self.counter
    }
}

impl RngCore for SimpleRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u32_internal()
    }

    fn next_u64(&mut self) -> u64 {
        let high = self.next_u32_internal() as u64;
        let low = self.next_u32_internal() as u64;
        (high << 32) | low
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let random = self.next_u32_internal();
            let bytes = random.to_le_bytes();
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte = bytes[i];
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// Mark our RNG as cryptographically secure (for demo purposes)
impl CryptoRng for SimpleRng {}

// Helper function to convert bytes to hex string
fn bytes_to_hex_string<'a>(bytes: &[u8], buffer: &'a mut [u8]) -> &'a str {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut i = 0;
    for &byte in bytes {
        buffer[i] = HEX_CHARS[(byte >> 4) as usize];
        buffer[i + 1] = HEX_CHARS[(byte & 0x0F) as usize];
        i += 2;
    }
    core::str::from_utf8(&buffer[..i]).unwrap()
}

// Derive Ethereum address from public key
fn derive_ethereum_address(public_key: &PublicKey) -> [u8; 20] {
    // Get uncompressed public key (65 bytes: 0x04 + x + y)
    let uncompressed = public_key.to_encoded_point(false);
    let public_key_bytes = uncompressed.as_bytes();

    // Ethereum address is last 20 bytes of Keccak256 hash of public key (without 0x04 prefix)
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]); // Skip the 0x04 prefix
    let hash = hasher.finalize();

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]); // Take last 20 bytes
    address
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

        let signing_key = SigningKey::random(&mut rng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = PublicKey::from(&verifying_key);

        // Derive Ethereum address
        let eth_address = derive_ethereum_address(&public_key);

        rprintln!("✓ Ethereum key pair generated successfully");

        // Print private key (32 bytes = 64 hex chars)
        let private_key_bytes = signing_key.to_bytes();
        let mut priv_hex_buffer = [0u8; 64];
        let priv_hex = bytes_to_hex_string(&private_key_bytes, &mut priv_hex_buffer);
        rprintln!("  Private Key: 0x{}", priv_hex);

        // Print public key (compressed - 33 bytes = 66 hex chars)
        let public_key_compressed = public_key.to_encoded_point(true);
        let pub_bytes = public_key_compressed.as_bytes();
        let mut pub_hex_buffer = [0u8; 66];
        let pub_hex = bytes_to_hex_string(pub_bytes, &mut pub_hex_buffer);
        rprintln!("  Public Key (compressed): 0x{}", pub_hex);

        // Print Ethereum address (20 bytes = 40 hex chars)
        let mut addr_hex_buffer = [0u8; 40];
        let addr_hex = bytes_to_hex_string(&eth_address, &mut addr_hex_buffer);
        rprintln!("  Ethereum Address: 0x{}", addr_hex);

        let step1_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step1_cycles.wrapping_sub(start_cycles));

        // Step 2: Create message and hash it
        rprintln!("\n[2/5] Creating message to sign...");
        let message = b"Hello, Ethereum ECDSA on STM32!";
        rprintln!("  Message: {:?}", core::str::from_utf8(message).unwrap());

        // Hash the message using Keccak256 (Ethereum standard)
        let mut hasher = Keccak256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        rprintln!("  Keccak256 hash: {:02X?}...", &message_hash[..8]);

        let step2_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step2_cycles.wrapping_sub(step1_cycles));

        // Step 3: Sign the message
        rprintln!("\n[3/5] Signing message with ECDSA...");
        let signature: Signature = signing_key.sign(message);
        rprintln!("✓ Signature generated successfully");
        rprintln!("  Signature (r,s): {:02X?}...", &signature.to_bytes()[..8]);

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
        match verifying_key.verify(message, &signature) {
            Ok(_) => {
                rprintln!("✓ Signature verification SUCCESSFUL!");

                let step5_cycles = cortex_m::peripheral::DWT::cycle_count();
                rprintln!("  Cycles: {}", step5_cycles.wrapping_sub(step4_cycles));

                // Blink LED to show verification complete
                for _i in 1..=5 {
                    led.set_low(); // LED on
                    delay_ms(50);
                    led.set_high(); // LED off
                    delay_ms(50);
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
                for _ in 0..20 {
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
