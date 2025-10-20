# STM32 Tests

A collection of STM32F411 (BlackPill) test programs and utilities.

## Project Structure

```
stm32-tests/
├── src/
│   ├── lib.rs              # Main library exposing utilities
│   ├── bin/                # Multiple binary targets
│   │   └── counter.rs      # Loop counter/speed test binary
│   └── utils/              # Shared utility modules
│       ├── mod.rs          # Utils module definition
│       └── oled.rs         # SSD1306 OLED display driver
├── tests/                  # Test workspace member
├── Cargo.toml              # Main workspace configuration
└── memory.x                # Linker script for STM32F411
```

## Binaries

### counter
A simple loop counter and speed test that uses a button to trigger counting loops and blinks an LED when complete.

**Hardware:**
- LED on PC13
- Button on PA0

**Build & Flash:**
```bash
cargo build --bin counter --release
probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/counter
```

## Utilities

### OLED Display Driver (`utils/oled`)
Minimal SSD1306 OLED display driver for STM32 using SPI interface.

**Pin Configuration:**
- VCC → 3V3
- GND → GND
- D0 (SCK) → PA5 (SPI1_SCK)
- D1 (MOSI) → PA7 (SPI1_MOSI)
- CS → GND (tied low)
- DC → PB0 (GPIO)
- RES → PB1 (GPIO)

**Usage:**
```rust
use stm32_tests::OledDisplay;

let mut display = OledDisplay::new(spi, dc_pin, rst_pin)?;
display.clear()?;
display.draw_text("Hello!", 10, 2)?;
```

## Adding New Binaries

To add a new binary, simply create a new file in `src/bin/` (e.g., `src/bin/my_app.rs`). Cargo will automatically discover it - no need to modify `Cargo.toml`!

**Example:**
```rust
#![no_std]
#![no_main]

use panic_rtt_target as _;
use cortex_m_rt::entry;
use stm32_tests::OledDisplay; // Import from library

#[entry]
fn main() -> ! {
    // Your code here
    loop {}
}
```

**Build & Flash:**
```bash
cargo build --bin my_app --release
probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/my_app
```

## Adding New Utilities

To add new utilities (crypto, UI components, etc.):

1. Create a new module in `src/utils/` (e.g., `src/utils/crypto.rs`)
2. Add the module to `src/utils/mod.rs`:
```rust
pub mod crypto;
```
3. Optionally re-export commonly used items in `src/lib.rs`:
```rust
pub use utils::crypto::SomeFunction;
```

## Development

**Build library:**
```bash
cargo build --lib
```

**Build specific binary:**
```bash
cargo build --bin counter --release
```

**List all binaries:**
```bash
cargo build --bins --release
```

## Hardware

- **Board:** STM32F411CEUx (WeAct BlackPill)
- **MCU:** STM32F411CEU6
- **Flash:** 512KB
- **RAM:** 128KB
- **Clock:** 100MHz (configurable)

## License

This project is for testing and educational purposes.