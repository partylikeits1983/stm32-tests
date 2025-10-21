//! Falcon512 post-quantum signature utilities for STM32

extern crate alloc;
use alloc::vec::Vec;

use miden_crypto::dsa::rpo_falcon512::{PublicKey, SecretKey, Signature};
use miden_crypto::{Felt, Word};

/// Falcon512 key pair structure
pub struct Falcon512KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl Falcon512KeyPair {
    /// Generate a new Falcon512 key pair using the provided RNG
    pub fn generate<R: rand::Rng>(rng: &mut R) -> Self {
        let secret_key = SecretKey::with_rng(rng);
        let public_key = secret_key.public_key();

        Falcon512KeyPair {
            secret_key,
            public_key,
        }
    }

    /// Sign a message using Falcon512
    /// Message is hashed to a Word before signing
    pub fn sign<R: rand::Rng>(&self, message: &[u8], rng: &mut R) -> Signature {
        // Hash message to Word
        let message_word = Self::hash_message_to_word(message);
        self.secret_key.sign_with_rng(message_word, rng)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let message_word = Self::hash_message_to_word(message);
        self.public_key.verify(message_word, signature)
    }

    /// Get the public key as a Word (4 field elements)
    pub fn public_key_word(&self) -> Word {
        self.public_key.into()
    }

    /// Simple hash function to convert message bytes to Word
    /// This is a basic implementation - in production you'd use a proper hash
    fn hash_message_to_word(message: &[u8]) -> Word {
        use miden_crypto::hash::rpo::Rpo256;

        // Convert message bytes to field elements
        let mut elements = Vec::new();
        for chunk in message.chunks(8) {
            let mut val: u64 = 0;
            for (j, &byte) in chunk.iter().enumerate() {
                val |= (byte as u64) << (j * 8);
            }
            elements.push(Felt::new(val));
        }

        // Hash the elements and convert to Word
        let digest = Rpo256::hash_elements(&elements);
        digest.into()
    }
}
