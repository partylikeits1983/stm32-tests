# Groth16 Proof Generation for STM32

This directory contains the Circom circuit and tooling for generating Groth16 zero-knowledge proofs that can be verified on STM32 microcontrollers.

## Overview

The system proves knowledge of a preimage `x` such that:
1. `leaf = Poseidon(x)` (hash of the preimage)
2. The `leaf` is included in a 20-level Incremental Merkle Tree with a public `root`

**Circuit Stats:**
- Constraints: 5,073
- Curve: BN254 (bn128)
- Hash: Poseidon (ZK-friendly)
- Tree depth: 20 levels

## Prerequisites

### macOS Setup

```bash
# Install Node.js and npm
brew install node

# Install Circom v2
npm install -g circom@latest

# Install snarkjs
npm install -g snarkjs@latest

# Install circomlib (for Poseidon hash)
cd circom
npm install circomlib@latest
```

### Rust Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM Cortex-M target
rustup target add thumbv7em-none-eabihf

# Install probe-rs for flashing
cargo install probe-rs --features cli
```

## Step-by-Step Guide

### 1. Compile the Circuit

```bash
# Compile the Circom circuit with depth=20
circom imt_preimage.circom --r1cs --wasm --sym -DDEPTH=20 -l node_modules/circomlib/circuits

# Check circuit info
snarkjs r1cs info imt_preimage.r1cs
```

**Output:** `imt_preimage.r1cs`, `imt_preimage.sym`, `imt_preimage_js/`

### 2. Trusted Setup (Groth16)

Run the fast setup script (uses power 13 for demo):

```bash
chmod +x setup_fast.sh
./setup_fast.sh
```

Or manually:

```bash
# Phase 1: Powers of Tau ceremony
snarkjs powersoftau new bn128 13 pot13_0000.ptau -v
snarkjs powersoftau contribute pot13_0000.ptau pot13_final.ptau -v

# Phase 2: Circuit-specific setup
snarkjs groth16 setup imt_preimage.r1cs pot13_final.ptau imt_preimage_0000.zkey
snarkjs zkey contribute imt_preimage_0000.zkey imt_preimage_final.zkey -v

# Export verification key
snarkjs zkey export verificationkey imt_preimage_final.zkey vk.json
```

**Output:** `imt_preimage_final.zkey`, `vk.json`

### 3. Prepare Test Input

Create `input.json` with your test data:

```json
{
  "root": "5719944538356554403817391916218209963465482684641156206505489751898082045988",
  "x": "12345",
  "path_elements": [
    "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
    "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"
  ],
  "path_index_bits": [
    "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
    "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"
  ]
}
```

**Helper:** Use `compute_root.js` to calculate the correct root:

```bash
node compute_root.js
```

### 4. Generate Proof

Run the proof generation script:

```bash
chmod +x generate_proof.sh
./generate_proof.sh
```

Or manually:

```bash
# Generate witness
node imt_preimage_js/generate_witness.js imt_preimage_js/imt_preimage.wasm input.json witness.wtns

# Generate proof
snarkjs groth16 prove imt_preimage_final.zkey witness.wtns proof.json public.json

# Verify proof locally
snarkjs groth16 verify vk.json public.json proof.json
```

**Output:** `proof.json`, `public.json`, `witness.wtns`

Expected output: `[INFO]  snarkJS: OK!`

### 5. Convert to Rust Constants

Build and run the converter:

```bash
cd converter
cargo build --release
cargo run --release
```

**Output:** `../../src/utils/vk_proof.rs`

This generates Rust constants from the JSON files that can be embedded in the STM32 firmware.

### 6. Build STM32 Firmware

```bash
cd ../..  # Back to project root
cargo build --release --bin groth16_demo
```

### 7. Flash to STM32

```bash
probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/groth16_demo
```

### 8. Test on Hardware

1. **Press the button** (PA0) to start verification
2. **LED turns on** (PC13) - verification in progress
3. **LED blinks**:
   - **Fast (120ms)**: Proof is VALID ✓
   - **Slow (400ms)**: Proof is INVALID ✗
4. Press button again to verify another time

## Performance

On STM32F411 @ 84MHz:
- **VK Preparation**: ~3.5 seconds (one pairing)
- **Proof Verification**: ~1.9 seconds (three pairings)


## File Structure

```
circom/
├── imt_preimage.circom          # Circuit definition
├── setup_fast.sh                # Quick trusted setup (power 13)
├── generate_proof.sh            # Proof generation script
├── compute_root.js              # Helper to compute Merkle root
├── input.json                   # Test input data
├── vk.json                      # Verification key (generated)
├── proof.json                   # Proof (generated)
├── public.json                  # Public inputs (generated)
└── converter/                   # Rust converter tool
    ├── Cargo.toml
    └── src/main.rs
```

## Troubleshooting

### Circuit Compilation Errors

**Error:** `Non quadratic constraints are not allowed!`

**Solution:** The circuit uses intermediate signals to avoid non-quadratic constraints. Make sure you're using Circom v2.1.5+.

### Proof Verification Fails

**Error:** Proof verifies with snarkjs but fails on STM32

**Solution:** 
1. Check that the converter ran successfully
2. Verify `vk_proof.rs` was generated
3. Rebuild the firmware: `cargo clean && cargo build --release`

### Out of Memory on STM32

**Error:** Allocation errors during verification

**Solution:**
- Increase heap size in `src/bin/groth16_demo.rs` (currently 64KB)
- Use a larger STM32 (F7/H7 series)
- Reduce circuit complexity

### Slow Verification

**Issue:** Verification takes too long

**Solutions:**
- Use faster MCU (STM32H7 @ 400MHz)
- Pre-compute `e_alpha_beta` (already done)
- Consider using a different proof system (PLONK, STARKs)

## Advanced Usage

### Custom Circuit Depth

To change the Merkle tree depth:

```bash
# Compile with different depth (e.g., 10 levels)
circom imt_preimage.circom --r1cs --wasm --sym -DDEPTH=10 -l node_modules/circomlib/circuits

# Update input.json to have 10 path elements instead of 20
```

### Production Setup

For production use, perform a proper Powers of Tau ceremony:

```bash
# Use power 16 for production (2^16 = 65,536 constraints)
snarkjs powersoftau new bn128 16 pot16_0000.ptau -v

# Multiple contributions for security
snarkjs powersoftau contribute pot16_0000.ptau pot16_0001.ptau -v
snarkjs powersoftau contribute pot16_0001.ptau pot16_final.ptau -v

# Continue with Phase 2...
```

### Different Hash Functions

To use a different hash function, modify the circuit:

```circom
// Replace Poseidon with MiMC
include "circomlib/mimc.circom";

component hash = MiMCSponge(1, 220, 1);
hash.ins[0] <== x;
hash.k <== 0;
signal leaf <== hash.outs[0];
```

## Resources

- [Circom Documentation](https://docs.circom.io/)
- [snarkjs Documentation](https://github.com/iden3/snarkjs)
- [Arkworks Documentation](https://arkworks.rs/)
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)
- [Poseidon Hash](https://www.poseidon-hash.info/)

## License

This project is part of the STM32 tests repository.