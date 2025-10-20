#![no_std]
#![no_main]

use panic_rtt_target as _;
use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use stm32f4xx_hal::{
    pac,
    prelude::*,
};

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 2000) {
        cortex_m::asm::nop();
    }
}

// Target count - change this value to test different loop counts
const TARGET_COUNT: u64 = 10_000_000;

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== STM32 Loop Speed Test ===");
    rprintln!("Target count: {}", TARGET_COUNT);
    rprintln!("Press button to start counting loop");
    
    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    
    // Get RCC peripheral for enabling GPIO clocks
    let mut rcc = dp.RCC;

    // Setup GPIOC (PC13 LED)
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();
    
    // Setup GPIOA (PA0 Button)
    let gpioa = dp.GPIOA.split(&mut rcc);
    let button = gpioa.pa0.into_pull_up_input();
    
    // Turn LED off initially (LED is active low on STM32 Blackpill)
    led.set_high();
    
    rprintln!("Ready! Press button to start...");
    
    loop {
        // Wait for button press
        while button.is_high() {
            // Wait for button to be pressed
        }
        
        rprintln!("Button pressed! Starting count loop...");
        
        // Start counting loop - this is the speed test
        // Use volatile operations to prevent compiler optimization
        let mut count: u64 = 0;
        while count < TARGET_COUNT {
            count += 1;
            // Use black_box to prevent the compiler from optimizing away the loop
            core::hint::black_box(&count);
        }
        
        rprintln!("Count reached {}! Blinking LED...", count);
        
        // Blink LED 3 times quickly
        for i in 1..=5 {
            rprintln!("Blink {}/3", i);
            led.set_low();  // LED on
            delay_ms(75);
            led.set_high(); // LED off
            delay_ms(75);
        }
        
        rprintln!("Test complete. Press button to run again.");
        
        // Wait for button release before next iteration
        while button.is_low() {
            // Wait for button to be released
        }
        
        delay_ms(200); // Debounce delay
    }
}
