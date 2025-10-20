# ECDSA Test Program

This is a host-side test program to verify ECDSA signature generation and verification works correctly before deploying to the STM32.

## Running the Test

```bash
cd tests
cargo run
```

The test will automatically build for your host system (macOS, Linux, etc.).

## What it does

1. Generates a random ECDSA key pair using P-256 curve
2. Signs the message "Hello, STM32!" 
3. Verifies the signature
4. Tests that verification fails with a wrong message

This confirms the ECDSA implementation works correctly before flashing to the microcontroller.