#!/bin/bash
# Setup script for Circom circuit compilation and Groth16 trusted setup

set -e

echo "=== Circom IMT Preimage Groth16 Setup ==="
echo ""

# Check if circom is installed
if ! command -v circom &> /dev/null; then
    echo "Error: circom not found. Please install it first:"
    echo "  npm install -g circom@latest"
    exit 1
fi

# Check if snarkjs is installed
if ! command -v snarkjs &> /dev/null; then
    echo "Error: snarkjs not found. Please install it first:"
    echo "  npm install -g snarkjs@latest"
    exit 1
fi

# Install circomlib if not present
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
echo "Step 3: Powers of Tau ceremony (Phase 1)..."
if [ ! -f "pot16_final.ptau" ]; then
    echo "  Creating new Powers of Tau..."
    snarkjs powersoftau new bn128 16 pot16_0000.ptau -v
    snarkjs powersoftau contribute pot16_0000.ptau pot16_0001.ptau --name="First contribution" -v -e="random entropy"
    snarkjs powersoftau prepare phase2 pot16_0001.ptau pot16_final.ptau -v
else
    echo "  Using existing pot16_final.ptau"
fi

echo ""
echo "Step 4: Groth16 setup (Phase 2)..."
snarkjs groth16 setup imt_preimage.r1cs pot16_final.ptau imt_preimage_0000.zkey
snarkjs zkey contribute imt_preimage_0000.zkey imt_preimage_final.zkey --name="Circuit contribution" -v -e="more random entropy"

echo ""
echo "Step 5: Export verification key..."
snarkjs zkey export verificationkey imt_preimage_final.zkey vk.json

echo ""
echo "=== Setup Complete ==="
echo "Files generated:"
echo "  - imt_preimage.r1cs (circuit constraints)"
echo "  - imt_preimage_js/ (witness generator)"
echo "  - imt_preimage_final.zkey (proving key)"
echo "  - vk.json (verification key)"
echo ""
echo "Next steps:"
echo "  1. Create input.json with your witness data"
echo "  2. Run: ./generate_proof.sh"