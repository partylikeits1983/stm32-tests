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
    echo "Please run ./setup_fast.sh first"
    exit 1
fi

echo "Circuit Information:"
echo "-------------------"
snarkjs r1cs info imt_preimage.r1cs
echo ""

echo "Step 1: Generating witness..."
echo "----------------------------"
WITNESS_START=$(date +%s)
/usr/bin/time -l node imt_preimage_js/generate_witness.js imt_preimage_js/imt_preimage.wasm input.json witness.wtns 2>&1 | tee /tmp/witness_time.txt
WITNESS_END=$(date +%s)
WITNESS_TIME=$((WITNESS_END - WITNESS_START))

# Extract memory usage from time output
WITNESS_MEM=$(grep "maximum resident set size" /tmp/witness_time.txt | awk '{print $1}')
WITNESS_MEM_MB=$(echo "scale=2; $WITNESS_MEM / 1024 / 1024" | bc)

echo ""
echo "✓ Witness generated in ${WITNESS_TIME}s"
echo "  Peak memory: ${WITNESS_MEM_MB} MB"
echo ""

echo "Step 2: Generating proof..."
echo "---------------------------"
PROVE_START=$(date +%s)
/usr/bin/time -l snarkjs groth16 prove imt_preimage_final.zkey witness.wtns proof.json public.json 2>&1 | tee /tmp/prove_time.txt
PROVE_END=$(date +%s)
PROVE_TIME=$((PROVE_END - PROVE_START))

# Extract memory usage from time output
PROVE_MEM=$(grep "maximum resident set size" /tmp/prove_time.txt | awk '{print $1}')
PROVE_MEM_MB=$(echo "scale=2; $PROVE_MEM / 1024 / 1024" | bc)

echo ""
echo "✓ Proof generated in ${PROVE_TIME}s"
echo "  Peak memory: ${PROVE_MEM_MB} MB"
echo ""

echo "Step 3: Verifying proof locally..."
echo "-----------------------------------"
VERIFY_START=$(date +%s)
snarkjs groth16 verify vk.json public.json proof.json
VERIFY_END=$(date +%s)
VERIFY_TIME=$((VERIFY_END - VERIFY_START))

echo ""
echo "✓ Proof verified in ${VERIFY_TIME}s"
echo ""

# Clean up temp files
rm -f /tmp/witness_time.txt /tmp/prove_time.txt

echo "=== Performance Summary ==="
echo "Witness generation: ${WITNESS_TIME}s (${WITNESS_MEM_MB} MB)"
echo "Proof generation:   ${PROVE_TIME}s (${PROVE_MEM_MB} MB)"
echo "Proof verification: ${VERIFY_TIME}s"
echo "Total time:         $((WITNESS_TIME + PROVE_TIME + VERIFY_TIME))s"
echo ""

echo "=== Files Generated ==="
echo "  - witness.wtns (witness)"
echo "  - proof.json (Groth16 proof)"
echo "  - public.json (public inputs)"
echo ""
echo "Next step:"
echo "  cd converter && cargo run --release"
echo "  This will generate ../src/utils/vk_proof.rs for your STM32"