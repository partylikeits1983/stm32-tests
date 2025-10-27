#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import Falcon512 utilities from pqcrypto-falcon crate
use pqcrypto_traits::sign::SignedMessage;
use stm32_tests::utils::falcon::Falcon512KeyPair;

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== Falcon512 Key Generation, Signing & Verification Example ===");
    rprintln!("Using pqcrypto-falcon crate (no-std compatible)");

    // Initialize heap for allocations
    const HEAP_SIZE: usize = 96 * 1024; // 96KB heap
    static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        stm32_tests::ALLOCATOR.init(
            core::ptr::addr_of!(HEAP_MEM) as *const _ as usize,
            HEAP_SIZE,
        )
    }

    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC;

    // Setup LED on PC13
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();

    rprintln!("\n=== Step 1: Key Generation ===");
    rprintln!("Generating Falcon512 keypair...");
    led.set_low(); // LED on during key generation

    // Generate keypair using pqcrypto-falcon
    let keypair = Falcon512KeyPair::generate();

    led.set_high(); // LED off
    rprintln!("✓ Keypair generated successfully!");
    rprintln!(
        "  Public key size: {} bytes",
        keypair.public_key_bytes().len()
    );
    rprintln!(
        "  Secret key size: {} bytes",
        keypair.secret_key_bytes().len()
    );

    rprintln!("\n=== Step 2: Signing ===");
    let message = b"Hello, Falcon512 Post-Quantum Crypto on STM32!";
    rprintln!(
        "Message to sign: {:?}",
        core::str::from_utf8(message).unwrap()
    );

    led.set_low(); // LED on during signing

    // Sign the message
    let signed_message = keypair.sign(message);

    led.set_high(); // LED off
    rprintln!("✓ Message signed successfully!");
    rprintln!(
        "  Signed message size: {} bytes",
        signed_message.as_bytes().len()
    );

    rprintln!("\n=== Step 3: Verification ===");
    led.set_low(); // LED on during verification

    // Verify the signature and recover the message
    match keypair.verify(&signed_message) {
        Ok(verified_msg) => {
            led.set_high(); // LED off
            rprintln!("✓ Signature verification SUCCESSFUL!");
            rprintln!(
                "  Verified message: {:?}",
                core::str::from_utf8(&verified_msg).unwrap()
            );

            // Check if the verified message matches the original
            if verified_msg == message {
                rprintln!("✓ Verified message matches original!");

                // Keep LED on to indicate success
                led.set_low();
                rprintln!("\n=== Success! LED will stay ON ===");
            } else {
                rprintln!("✗ Message mismatch!");
            }
        }
        Err(e) => {
            led.set_high();
            rprintln!("✗ Signature verification FAILED: {}", e);
        }
    }

    rprintln!("\n=== Falcon512 Information ===");
    rprintln!("Algorithm: Falcon512 (lattice-based)");
    rprintln!("Security Level: NIST Level 1 (128-bit quantum security)");
    rprintln!("Post-Quantum: Resistant to quantum computer attacks");
    rprintln!(
        "Public Key: {} bytes",
        stm32_tests::utils::falcon::public_key_bytes_len()
    );
    rprintln!(
        "Secret Key: {} bytes",
        stm32_tests::utils::falcon::secret_key_bytes_len()
    );
    rprintln!(
        "Max Signature: {} bytes",
        stm32_tests::utils::falcon::signature_bytes_len()
    );

    rprintln!("\n=== Example Complete ===");

    // Infinite loop
    loop {
        cortex_m::asm::nop();
    }
}
