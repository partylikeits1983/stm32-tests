#!/bin/bash
# Generate a Groth16 proof from witness input

set -e

echo "=== Generating Groth16 Proof ==="
echo ""

if [ ! -f "input.json" ]; then
    echo "Error: input.json not found!"
    echo "Please create input.json with your witness data."
    echo ""
    echo "Example format:"
    echo '{'
    echo '  "root": "12345678901234567890",  // public'
    echo '  "x": "42",                        // private preimage'
    echo '  "path_elements": ["...", "..."], // 20 elements'
    echo '  "path_index_bits": [0, 1, ...]   // 20 bits (0 or 1)'
    echo '}'
    exit 1
fi

if [ ! -f "imt_preimage_final.zkey" ]; then
    echo "Error: imt_preimage_final.zkey not found!"
    echo "Please run ./setup.sh first"
    exit 1
fi

echo "Step 1: Generating witness..."
node imt_preimage_js/generate_witness.js imt_preimage_js/imt_preimage.wasm input.json witness.wtns

echo ""
echo "Step 2: Generating proof..."
snarkjs groth16 prove imt_preimage_final.zkey witness.wtns proof.json public.json

echo ""
echo "Step 3: Verifying proof locally..."
snarkjs groth16 verify vk.json public.json proof.json

echo ""
echo "=== Proof Generation Complete ==="
echo "Files generated:"
echo "  - witness.wtns (witness)"
echo "  - proof.json (Groth16 proof)"
echo "  - public.json (public inputs)"
echo ""
echo "Next step:"
echo "  Run: node emit_rust_from_snarkjs.js"
echo "  This will generate vk_proof.rs for your STM32"