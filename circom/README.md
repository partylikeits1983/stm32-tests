# Groth16 Proof Generation for STM32

This directory contains the Circom circuit and tooling for generating Groth16 zero-knowledge proofs that can be verified on STM32 microcontrollers.

## Overview

The system proves knowledge of a preimage `x` such that:
1. `leaf = Poseidon(x)` (hash of the preimage)
2. The `leaf` is included in a 20-level Incremental Merkle Tree with a public `root`

**Circuit Stats:**
- Constraints: 5,073
- Curve: BN254 (bn128)
- Hash Function: Poseidon (ZK-friendly)
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

## Quick Start (Automated)

If you just want to get started quickly:

```bash
# 1. Install dependencies
npm install circomlib

# 2. Run setup (one-time, generates keys)
./setup_fast.sh

# 3. Generate proof
./generate_proof.sh

# 4. Convert to Rust
cd converter && cargo run --release && cd ..

# 5. Build and flash (from project root)
cd .. && cargo build --release --bin groth16_demo
probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/groth16_demo
```

## Step-by-Step Guide (Manual)

### Step 1: Compile the Circuit

```bash
# Compile the Circom circuit with depth=20
circom imt_preimage.circom --r1cs --wasm --sym -DDEPTH=20 -l node_modules/circomlib/circuits

# Check circuit info (shows constraints, wires, etc.)
snarkjs r1cs info imt_preimage.r1cs
```

**Output:** 
- `imt_preimage.r1cs` (constraint system)
- `imt_preimage.sym` (symbol table)
- `imt_preimage_js/` (witness generator)

**Expected:** `# of Constraints: 5073`

### Step 2: Powers of Tau Ceremony (One-Time Setup)

This step only needs to be done once. It generates the common reference string (CRS) for the proof system.

```bash
# Generate initial Powers of Tau (power 13 = 2^13 = 8192 constraints)
snarkjs powersoftau new bn128 13 pot13_0000.ptau -v

# Contribute randomness (can be done multiple times for security)
snarkjs powersoftau contribute pot13_0000.ptau pot13_final.ptau -v
```

**Output:** `pot13_final.ptau` (Powers of Tau file)

**Note:** For production, use power 16 or higher and multiple contributions.

### Step 3: Circuit-Specific Setup (One-Time per Circuit)

Generate the proving and verification keys for this specific circuit:

```bash
# Phase 2: Circuit-specific setup
snarkjs groth16 setup imt_preimage.r1cs pot13_final.ptau imt_preimage_0000.zkey

# Contribute randomness to the circuit-specific key
snarkjs zkey contribute imt_preimage_0000.zkey imt_preimage_final.zkey -v

# Export verification key (needed for verification)
snarkjs zkey export verificationkey imt_preimage_final.zkey vk.json
```

**Output:**
- `imt_preimage_final.zkey` (proving key)
- `vk.json` (verification key)

### Step 4: Prepare Test Input

Create `input.json` with your witness data:

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

**Helper:** Use `compute_root.js` to calculate the correct root for your inputs:

```bash
node compute_root.js
```

### Step 5: Generate Witness

Compute the witness (all intermediate values) from your input:

```bash
node imt_preimage_js/generate_witness.js \
     imt_preimage_js/imt_preimage.wasm \
     input.json \
     witness.wtns
```

**Output:** `witness.wtns` (witness file)

### Step 6: Generate Proof

Create the zero-knowledge proof:

```bash
snarkjs groth16 prove imt_preimage_final.zkey witness.wtns proof.json public.json
```

**Output:**
- `proof.json` (the Groth16 proof)
- `public.json` (public inputs)

### Step 7: Verify Proof Locally

Before deploying to STM32, verify the proof works:

```bash
snarkjs groth16 verify vk.json public.json proof.json
```

**Expected output:** `[INFO]  snarkJS: OK!`

### Step 8: Convert to Rust Constants

Convert the JSON files to embedded Rust code:

```bash
cd converter
cargo build --release
cargo run --release
```

**Output:** `../../src/utils/vk_proof.rs` (Rust constants)

This file contains:
- Verification key as Rust structs
- Sample proof as Rust structs
- All using arkworks types compatible with no_std

### Step 9: Build STM32 Firmware

```bash
cd ../..  # Back to project root
cargo build --release --bin groth16_demo
```

**Output:** `target/thumbv7em-none-eabihf/release/groth16_demo`

### Step 10: Flash to STM32

```bash
probe-rs run --chip STM32F411CEUx target/thumbv7em-none-eabihf/release/groth16_demo
```

### Step 11: Test on Hardware

1. **Press the button** (PA0) to start verification
2. **LED turns on** (PC13) - verification in progress
3. **Wait ~2 seconds** for verification to complete
4. **LED blinks**:
   - **Fast (120ms intervals)**: Proof is VALID ✓
   - **Slow (400ms intervals)**: Proof is INVALID ✗
5. Press button again to verify another time

## Performance

On STM32F411 @ 84MHz:
- **VK Preparation**: ~3.5 seconds (one pairing, done once at startup)
- **Proof Verification**: ~1.9 seconds (three pairings)
- **Total per verification**: ~2.2 seconds

## File Structure

```
circom/
├── imt_preimage.circom          # Circuit definition
├── setup_fast.sh                # Automated setup script
├── generate_proof.sh            # Automated proof generation
├── compute_root.js              # Helper to compute Merkle root
├── input.json                   # Your test input data
│
├── vk.json                      # Verification key (generated)
├── proof.json                   # Proof (generated)
├── public.json                  # Public inputs (generated)
│
└── converter/                   # Rust converter tool
    ├── Cargo.toml
    └── src/main.rs
```

## Workflow Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    ONE-TIME SETUP                            │
├─────────────────────────────────────────────────────────────┤
│ 1. Compile Circuit    → imt_preimage.r1cs                   │
│ 2. Powers of Tau      → pot13_final.ptau                    │
│ 3. Circuit Setup      → imt_preimage_final.zkey, vk.json    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              FOR EACH PROOF (Prover Side)                    │
├─────────────────────────────────────────────────────────────┤
│ 4. Create input.json  → Your witness data                   │
│ 5. Generate Witness   → witness.wtns                        │
│ 6. Generate Proof     → proof.json, public.json             │
│ 7. Verify Locally     → Check with snarkjs                  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│           DEPLOY TO STM32 (Verifier Side)                    │
├─────────────────────────────────────────────────────────────┤
│ 8. Convert to Rust    → vk_proof.rs                         │
│ 9. Build Firmware     → groth16_demo binary                 │
│ 10. Flash to STM32    → Upload to microcontroller           │
│ 11. Test              → Press button, watch LED             │
└─────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Circuit Compilation Errors

**Error:** `Non quadratic constraints are not allowed!`

**Solution:** The circuit uses intermediate signals to avoid non-quadratic constraints. Make sure you're using Circom v2.1.5+.

### Proof Verification Fails Locally

**Error:** `snarkjs groth16 verify` returns INVALID

**Possible causes:**
1. Wrong input data in `input.json`
2. Mismatched root value
3. Incorrect path_elements or path_index_bits

**Solution:** Use `compute_root.js` to calculate the correct root for your inputs.

### Proof Verifies Locally but Fails on STM32

**Error:** Proof verifies with snarkjs but fails on STM32

**Solution:** 
1. Check that the converter ran successfully
2. Verify `vk_proof.rs` was generated in `src/utils/`
3. Rebuild the firmware: `cargo clean && cargo build --release`
4. Make sure you're using the same `vk.json`, `proof.json`, and `public.json`

### Out of Memory on STM32

**Error:** Allocation errors during verification

**Solution:**
- Increase heap size in `src/bin/groth16_demo.rs` (currently 64KB)
- Use a larger STM32 (F7/H7 series with more RAM)
- Reduce circuit complexity (smaller tree depth)

### Slow Verification

**Issue:** Verification takes too long

**Solutions:**
- Use faster MCU (STM32H7 @ 400MHz instead of F4 @ 84MHz)
- Pre-compute `e_alpha_beta` (already done in current implementation)
- Consider using a different proof system (PLONK, STARKs)
- Reduce circuit size (fewer constraints)

### Powers of Tau File Too Small

**Error:** `Powers of Tau is too small for this circuit`

**Solution:** Use a larger power:
```bash
# For circuits with up to 65,536 constraints
snarkjs powersoftau new bn128 16 pot16_0000.ptau -v
```

## Advanced Usage

### Custom Circuit Depth

To change the Merkle tree depth:

```bash
# Compile with different depth (e.g., 10 levels)
circom imt_preimage.circom --r1cs --wasm --sym -DDEPTH=10 -l node_modules/circomlib/circuits

# Update input.json to have 10 path elements instead of 20
# Then follow steps 3-11 above
```

### Production Setup

For production use, perform a proper Powers of Tau ceremony:

```bash
# Use power 16 for production (2^16 = 65,536 constraints)
snarkjs powersoftau new bn128 16 pot16_0000.ptau -v

# Multiple contributions for security (can be done by different parties)
snarkjs powersoftau contribute pot16_0000.ptau pot16_0001.ptau -v
snarkjs powersoftau contribute pot16_0001.ptau pot16_0002.ptau -v
snarkjs powersoftau contribute pot16_0002.ptau pot16_final.ptau -v

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

Then recompile and follow the setup steps again.

## Resources

- [Circom Documentation](https://docs.circom.io/)
- [snarkjs Documentation](https://github.com/iden3/snarkjs)
- [Arkworks Documentation](https://arkworks.rs/)
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)
- [Poseidon Hash](https://www.poseidon-hash.info/)
- [Powers of Tau Ceremony](https://github.com/iden3/snarkjs#7-prepare-phase-2)

## License

This project is part of the STM32 tests repository.