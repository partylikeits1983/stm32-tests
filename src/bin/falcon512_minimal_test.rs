#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== Falcon512 Minimal Test ===");
    rprintln!("RTT initialized successfully!");

    // Initialize heap
    const HEAP_SIZE: usize = 96 * 1024;
    static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        stm32_tests::ALLOCATOR.init(
            core::ptr::addr_of!(HEAP_MEM) as *const _ as usize,
            HEAP_SIZE,
        )
    }
    rprintln!("Heap initialized: {} bytes", HEAP_SIZE);

    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC;

    // Setup LED
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();

    rprintln!("LED initialized");

    // Test LED
    for i in 0..5 {
        rprintln!("Blink {}", i + 1);
        led.set_low();
        cortex_m::asm::delay(8_000_000); // ~100ms at 84MHz
        led.set_high();
        cortex_m::asm::delay(8_000_000);
    }

    rprintln!("\n=== Testing Falcon512 Import ===");

    // Just test that we can import the module
    use stm32_tests::utils::falcon::{
        public_key_bytes_len, secret_key_bytes_len, signature_bytes_len,
    };

    rprintln!("Falcon512 constants:");
    rprintln!("  Public key size: {} bytes", public_key_bytes_len());
    rprintln!("  Secret key size: {} bytes", secret_key_bytes_len());
    rprintln!("  Max signature size: {} bytes", signature_bytes_len());

    rprintln!("\n=== Test Complete ===");
    rprintln!("Note: Key generation requires significant memory and time");
    rprintln!("LED will stay on");

    led.set_low(); // Keep LED on

    loop {
        cortex_m::asm::nop();
    }
}
