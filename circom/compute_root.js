// Compute the correct root for a simple test case
const { buildPoseidon } = require("circomlibjs");

async function main() {
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    
    // Our test preimage
    const x = 12345n;
    
    // Compute leaf = Poseidon([x])
    const leaf = poseidon([x]);
    const leafStr = F.toString(leaf);
    
    console.log("Preimage x:", x.toString());
    console.log("Leaf hash:", leafStr);
    
    // For a simple test: all siblings are 0, so we just hash up the tree
    // At each level: if bit=0, hash(current, 0); if bit=1, hash(0, current)
    // For simplicity, let's use all bits = 0 (left child at each level)
    
    let current = leaf;
    for (let i = 0; i < 20; i++) {
        // bit = 0, so hash(current, 0)
        current = poseidon([current, 0n]);
    }
    
    const root = F.toString(current);
    console.log("Root (20 levels, all siblings=0, all bits=0):", root);
    
    // Generate the input.json
    const input = {
        root: root,
        x: x.toString(),
        path_elements: Array(20).fill("0"),
        path_index_bits: Array(20).fill(0)
    };
    
    console.log("\nGenerated input.json:");
    console.log(JSON.stringify(input, null, 2));
}

main().catch(console.error);