//! Falcon512 post-quantum signature utilities for STM32
//! Using pqcrypto-falcon crate (no-std compatible)

extern crate alloc;
use alloc::vec::Vec;

use pqcrypto_falcon::falcon512::{
    detached_sign, keypair, open, sign, verify_detached_signature, DetachedSignature, PublicKey,
    SecretKey, SignedMessage,
};
use pqcrypto_traits::sign::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};

/// Falcon512 key pair structure
pub struct Falcon512KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl Falcon512KeyPair {
    /// Generate a new Falcon512 key pair
    /// Note: This uses the internal RNG from pqcrypto-falcon
    pub fn generate() -> Self {
        let (pk, sk) = keypair();

        Falcon512KeyPair {
            secret_key: sk,
            public_key: pk,
        }
    }

    /// Sign a message using Falcon512 (returns signed message)
    /// The signed message includes both the signature and the original message
    pub fn sign(&self, message: &[u8]) -> SignedMessage {
        sign(message, &self.secret_key)
    }

    /// Sign a message using Falcon512 (returns detached signature)
    /// The detached signature is separate from the message
    pub fn sign_detached(&self, message: &[u8]) -> DetachedSignature {
        detached_sign(message, &self.secret_key)
    }

    /// Verify a signed message and return the original message if valid
    pub fn verify(&self, signed_message: &SignedMessage) -> Result<Vec<u8>, &'static str> {
        open(signed_message, &self.public_key).map_err(|_| "Signature verification failed")
    }

    /// Verify a detached signature
    pub fn verify_detached(&self, signature: &DetachedSignature, message: &[u8]) -> bool {
        verify_detached_signature(signature, message, &self.public_key).is_ok()
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.public_key.as_bytes()
    }

    /// Get the secret key as bytes
    pub fn secret_key_bytes(&self) -> &[u8] {
        self.secret_key.as_bytes()
    }

    /// Create a keypair from existing key bytes
    pub fn from_bytes(pk_bytes: &[u8], sk_bytes: &[u8]) -> Result<Self, &'static str> {
        let public_key = PublicKey::from_bytes(pk_bytes).map_err(|_| "Invalid public key bytes")?;
        let secret_key = SecretKey::from_bytes(sk_bytes).map_err(|_| "Invalid secret key bytes")?;

        Ok(Falcon512KeyPair {
            secret_key,
            public_key,
        })
    }
}

/// Get the size of a Falcon512 public key in bytes
pub const fn public_key_bytes_len() -> usize {
    pqcrypto_falcon::falcon512::public_key_bytes()
}

/// Get the size of a Falcon512 secret key in bytes
pub const fn secret_key_bytes_len() -> usize {
    pqcrypto_falcon::falcon512::secret_key_bytes()
}

/// Get the maximum size of a Falcon512 signature in bytes
pub const fn signature_bytes_len() -> usize {
    pqcrypto_falcon::falcon512::signature_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = Falcon512KeyPair::generate();
        assert_eq!(keypair.public_key_bytes().len(), public_key_bytes_len());
        assert_eq!(keypair.secret_key_bytes().len(), secret_key_bytes_len());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = Falcon512KeyPair::generate();
        let message = b"Hello, Falcon512!";

        let signed_message = keypair.sign(message);
        let verified_message = keypair.verify(&signed_message).unwrap();

        assert_eq!(verified_message, message);
    }

    #[test]
    fn test_detached_sign_and_verify() {
        let keypair = Falcon512KeyPair::generate();
        let message = b"Hello, Falcon512 detached!";

        let signature = keypair.sign_detached(message);
        assert!(keypair.verify_detached(&signature, message));

        // Verify with wrong message should fail
        let wrong_message = b"Wrong message";
        assert!(!keypair.verify_detached(&signature, wrong_message));
    }

    #[test]
    fn test_from_bytes() {
        let keypair1 = Falcon512KeyPair::generate();
        let pk_bytes = keypair1.public_key_bytes();
        let sk_bytes = keypair1.secret_key_bytes();

        let keypair2 = Falcon512KeyPair::from_bytes(pk_bytes, sk_bytes).unwrap();

        // Sign with first keypair, verify with second
        let message = b"Test message";
        let signed_message = keypair1.sign(message);
        let verified_message = keypair2.verify(&signed_message).unwrap();

        assert_eq!(verified_message, message);
    }
}
