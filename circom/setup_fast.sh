#!/bin/bash
# Fast setup script for demo purposes (NOT for production!)

set -e

echo "=== Fast Circom Setup (Demo Only) ==="
echo ""

# Check dependencies
if ! command -v circom &> /dev/null; then
    echo "Error: circom not found. Install with: npm install -g circom@latest"
    exit 1
fi

if ! command -v snarkjs &> /dev/null; then
    echo "Error: snarkjs not found. Install with: npm install -g snarkjs@latest"
    exit 1
fi

# Install circomlib if needed
if [ ! -d "node_modules/circomlib" ]; then
    echo "Installing circomlib..."
    npm init -y
    npm install circomlib@latest
fi

echo "Step 1: Compiling circuit..."
circom imt_preimage.circom --r1cs --wasm --sym -l node_modules

echo ""
echo "Step 2: Circuit info..."
snarkjs r1cs info imt_preimage.r1cs

echo ""
echo "Step 3: Fast Powers of Tau (using smaller parameter for speed)..."
# Use power 13 instead of 16 - much faster!
# 2^13 = 8192 constraints (we have ~5000, so this is enough)
if [ ! -f "pot13_final.ptau" ]; then
    echo "  Creating Powers of Tau (2^13)..."
    snarkjs powersoftau new bn128 13 pot13_0000.ptau
    echo "  Preparing for phase 2..."
    snarkjs powersoftau prepare phase2 pot13_0000.ptau pot13_final.ptau
else
    echo "  Using existing pot13_final.ptau"
fi

echo ""
echo "Step 4: Groth16 setup..."
snarkjs groth16 setup imt_preimage.r1cs pot13_final.ptau imt_preimage_final.zkey

echo ""
echo "Step 5: Export verification key..."
snarkjs zkey export verificationkey imt_preimage_final.zkey vk.json

echo ""
echo "=== Setup Complete ==="
echo "Files generated:"
echo "  - imt_preimage.r1cs"
echo "  - imt_preimage_js/"
echo "  - imt_preimage_final.zkey"
echo "  - vk.json"
echo ""
echo "Next: Create input.json and run ./generate_proof.sh"