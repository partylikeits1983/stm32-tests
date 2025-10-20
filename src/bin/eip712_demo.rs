#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

// Import crypto utilities
use stm32_tests::utils::crypto::{
    bytes_to_hex_string, eip712_hash, sign_eip712, verify_eip712, Eip712Domain, EthereumKeyPair,
    MultisigTransaction, SimpleRng,
};

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
    rprintln!("=== STM32 EIP712 Typed Data Signing Demo ===");
    rprintln!("Using secp256k1 curve with EIP712 standard");
    rprintln!("Press button to generate keys and sign typed data");

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

    rprintln!("Ready! Press button to start EIP712 demo...");

    loop {
        // Wait for button press
        while button.is_high() {
            // Wait for button to be pressed
        }

        rprintln!("\n=== Starting EIP712 Demo ===");

        // Get start time (using DWT cycle counter for precise timing)
        let start_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("Start cycles: {}", start_cycles);

        // Step 1: Generate Ethereum key pair
        rprintln!("\n[1/5] Generating Ethereum key pair (secp256k1)...");
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

        // Print Ethereum address (20 bytes = 40 hex chars)
        let mut addr_hex_buffer = [0u8; 40];
        let addr_hex = bytes_to_hex_string(&keypair.address, &mut addr_hex_buffer);
        rprintln!("  Ethereum Address: 0x{}", addr_hex);

        let step1_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step1_cycles.wrapping_sub(start_cycles));

        // Step 2: Create EIP712 domain and typed data
        rprintln!("\n[2/5] Creating EIP712 typed data (Multisig Transaction)...");

        // Create EIP712 domain
        let verifying_contract = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
        ];
        let domain = Eip712Domain::new("MyMultisig", "1", 1, verifying_contract);

        rprintln!("  Domain:");
        rprintln!("    Name: {}", domain.name);
        rprintln!("    Version: {}", domain.version);
        rprintln!("    Chain ID: {}", domain.chain_id);
        let mut contract_hex_buffer = [0u8; 40];
        let contract_hex =
            bytes_to_hex_string(&domain.verifying_contract, &mut contract_hex_buffer);
        rprintln!("    Verifying Contract: 0x{}", contract_hex);

        // Create multisig transaction
        let to_address = [
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
        ];
        let tx_data = b"transfer(address,uint256)";
        let transaction = MultisigTransaction::new(to_address, 1000000000000000000, tx_data, 42);

        rprintln!("  Transaction:");
        let mut to_hex_buffer = [0u8; 40];
        let to_hex = bytes_to_hex_string(&transaction.to, &mut to_hex_buffer);
        rprintln!("    To: 0x{}", to_hex);
        rprintln!("    Value: {} wei", transaction.value);
        rprintln!(
            "    Data: {:?}",
            core::str::from_utf8(transaction.data).unwrap()
        );
        rprintln!("    Nonce: {}", transaction.nonce);

        // Compute struct hash
        let struct_hash = transaction.hash_struct();
        let mut struct_hash_buffer = [0u8; 64];
        let struct_hash_hex = bytes_to_hex_string(&struct_hash, &mut struct_hash_buffer);
        rprintln!("  Struct Hash: 0x{}", struct_hash_hex);

        // Compute EIP712 message hash
        let message_hash = eip712_hash(&domain, &struct_hash);
        let mut msg_hash_buffer = [0u8; 64];
        let msg_hash_hex = bytes_to_hex_string(&message_hash, &mut msg_hash_buffer);
        rprintln!("  EIP712 Message Hash: 0x{}", msg_hash_hex);

        let step2_cycles = cortex_m::peripheral::DWT::cycle_count();
        rprintln!("  Cycles: {}", step2_cycles.wrapping_sub(step1_cycles));

        // Step 3: Sign the typed data
        rprintln!("\n[3/5] Signing EIP712 typed data with ECDSA...");
        let signature = sign_eip712(&keypair, &domain, &struct_hash);
        rprintln!("✓ Signature generated successfully");
        let sig_bytes = signature.to_bytes();
        let mut sig_hex_buffer = [0u8; 128];
        let sig_hex = bytes_to_hex_string(&sig_bytes, &mut sig_hex_buffer);
        rprintln!("  Signature: 0x{}", sig_hex);

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
        rprintln!("\n[5/5] Verifying EIP712 signature...");
        match verify_eip712(&keypair, &domain, &struct_hash, &signature) {
            Ok(_) => {
                rprintln!("✓ EIP712 Signature verification SUCCESSFUL!");

                let step5_cycles = cortex_m::peripheral::DWT::cycle_count();
                rprintln!("  Cycles: {}", step5_cycles.wrapping_sub(step4_cycles));

                // Blink LED 3 times to show verification complete
                rprintln!("\nBlinking LED 3 times (verification complete)...");
                for i in 1..=3 {
                    rprintln!("  Blink {}/3", i);
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
                rprintln!("✗ EIP712 Signature verification FAILED!");
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
