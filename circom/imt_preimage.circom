pragma circom 2.1.5;

include "circomlib/circuits/poseidon.circom";

/**
 * Prove: know x (private) such that:
 *  leaf = Poseidon([x])
 *  and leaf is included in a Merkle tree with public root via (path_elements, path_index_bits)
 *
 * Public inputs: root
 * Private inputs: x, path_elements[], path_index_bits[]
 *
 * Conventions:
 *  - path_index_bits[i] = 0 means leaf/current is LEFT child at level i
 *  - path_index_bits[i] = 1 means RIGHT child
 *
 * Tree arity = 2, height = DEPTH (set compile-time with a template parameter).
 */
template PreimageInIMT(DEPTH) {
    // Public
    signal input root;

    // Private
    signal input x;
    signal input path_elements[DEPTH];
    signal input path_index_bits[DEPTH]; // each 0/1

    // Compute leaf hash
    component poseidon_leaf = Poseidon(1);
    poseidon_leaf.inputs[0] <== x;
    
    // Declare all signals outside the loop
    signal cur[DEPTH + 1];
    signal left[DEPTH];
    signal right[DEPTH];
    
    // Intermediate signals for quadratic constraints
    signal s[DEPTH];
    signal t[DEPTH];
    
    // Declare all components outside the loop
    component hashers[DEPTH];
    for (var i = 0; i < DEPTH; i++) {
        hashers[i] = Poseidon(2);
    }
    
    // Start with the leaf
    cur[0] <== poseidon_leaf.out;

    // Iterate up the tree
    for (var i = 0; i < DEPTH; i++) {
        // Constrain bits to be 0/1
        path_index_bits[i] * (path_index_bits[i] - 1) === 0;
        
        // Select left/right ordering based on path bit using quadratic constraints
        // if bit==0: left=cur[i], right=path_elements[i]
        // if bit==1: left=path_elements[i], right=cur[i]
        
        // s[i] = path_index_bits[i] * (path_elements[i] - cur[i])
        s[i] <== path_index_bits[i] * (path_elements[i] - cur[i]);
        // left[i] = cur[i] + s[i]
        left[i] <== cur[i] + s[i];
        
        // t[i] = path_index_bits[i] * (cur[i] - path_elements[i])
        t[i] <== path_index_bits[i] * (cur[i] - path_elements[i]);
        // right[i] = path_elements[i] + t[i]
        right[i] <== path_elements[i] + t[i];

        // Hash the pair
        hashers[i].inputs[0] <== left[i];
        hashers[i].inputs[1] <== right[i];
        cur[i + 1] <== hashers[i].out;
    }

    // Final equality: cur must equal public root
    cur[DEPTH] === root;
}

component main {public [root]} = PreimageInIMT(20);