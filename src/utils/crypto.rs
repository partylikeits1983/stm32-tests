//! Cryptographic utilities for Ethereum ECDSA operations

use k256::{
    ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey},
    elliptic_curve::{
        rand_core::{CryptoRng, RngCore},
        sec1::ToEncodedPoint,
    },
    PublicKey,
};
use sha3::{Digest, Keccak256};

/// Simple RNG implementation using hardware timer as entropy source
pub struct SimpleRng {
    state: u32,
    counter: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        SimpleRng {
            state: seed,
            counter: 0,
        }
    }

    fn next_u32_internal(&mut self) -> u32 {
        // Simple LCG (Linear Congruential Generator) with counter mixing
        self.counter = self.counter.wrapping_add(1);
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state ^ self.counter
    }
}

impl RngCore for SimpleRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u32_internal()
    }

    fn next_u64(&mut self) -> u64 {
        let high = self.next_u32_internal() as u64;
        let low = self.next_u32_internal() as u64;
        (high << 32) | low
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let random = self.next_u32_internal();
            let bytes = random.to_le_bytes();
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte = bytes[i];
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// Mark our RNG as cryptographically secure (for demo purposes)
impl CryptoRng for SimpleRng {}

/// Helper function to convert bytes to hex string
pub fn bytes_to_hex_string<'a>(bytes: &[u8], buffer: &'a mut [u8]) -> &'a str {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut i = 0;
    for &byte in bytes {
        buffer[i] = HEX_CHARS[(byte >> 4) as usize];
        buffer[i + 1] = HEX_CHARS[(byte & 0x0F) as usize];
        i += 2;
    }
    core::str::from_utf8(&buffer[..i]).unwrap()
}

/// Ethereum key pair structure
pub struct EthereumKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub public_key: PublicKey,
    pub address: [u8; 20],
}

impl EthereumKeyPair {
    /// Generate a new Ethereum key pair using the provided RNG
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let signing_key = SigningKey::random(rng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key = PublicKey::from(&verifying_key);
        let address = Self::derive_address(&public_key);

        EthereumKeyPair {
            signing_key,
            verifying_key,
            public_key,
            address,
        }
    }

    /// Derive Ethereum address from public key
    fn derive_address(public_key: &PublicKey) -> [u8; 20] {
        // Get uncompressed public key (65 bytes: 0x04 + x + y)
        let uncompressed = public_key.to_encoded_point(false);
        let public_key_bytes = uncompressed.as_bytes();

        // Ethereum address is last 20 bytes of Keccak256 hash of public key (without 0x04 prefix)
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip the 0x04 prefix
        let hash = hasher.finalize();

        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]); // Take last 20 bytes
        address
    }

    /// Sign a message using ECDSA
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ecdsa::Error> {
        self.verifying_key.verify(message, signature)
    }

    /// Get the private key bytes
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes().into()
    }

    /// Get the compressed public key bytes
    pub fn public_key_compressed_bytes(&self) -> [u8; 33] {
        let encoded = self.public_key.to_encoded_point(true);
        let bytes = encoded.as_bytes();
        let mut result = [0u8; 33];
        result.copy_from_slice(bytes);
        result
    }
}

/// Hash a message using Keccak256 (Ethereum standard)
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}
