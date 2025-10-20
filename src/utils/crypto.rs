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

/// EIP712 Domain structure
#[derive(Clone)]
pub struct Eip712Domain {
    pub name: &'static str,
    pub version: &'static str,
    pub chain_id: u64,
    pub verifying_contract: [u8; 20],
}

impl Eip712Domain {
    /// Create a new EIP712 domain
    pub fn new(
        name: &'static str,
        version: &'static str,
        chain_id: u64,
        verifying_contract: [u8; 20],
    ) -> Self {
        Self {
            name,
            version,
            chain_id,
            verifying_contract,
        }
    }

    /// Compute the domain separator hash
    pub fn hash_struct(&self) -> [u8; 32] {
        // EIP712Domain type hash
        let type_hash = keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );

        let name_hash = keccak256(self.name.as_bytes());
        let version_hash = keccak256(self.version.as_bytes());

        // Encode: typeHash || nameHash || versionHash || chainId || verifyingContract
        let mut encoded = [0u8; 160]; // 32 + 32 + 32 + 32 + 32 (padded address)
        encoded[0..32].copy_from_slice(&type_hash);
        encoded[32..64].copy_from_slice(&name_hash);
        encoded[64..96].copy_from_slice(&version_hash);

        // Encode chain_id as uint256 (32 bytes, big-endian)
        let chain_id_bytes = self.chain_id.to_be_bytes();
        encoded[120..128].copy_from_slice(&chain_id_bytes);

        // Encode address (20 bytes, left-padded to 32 bytes)
        encoded[140..160].copy_from_slice(&self.verifying_contract);

        keccak256(&encoded)
    }
}

/// EIP712 Multisig Transaction structure
#[derive(Clone)]
pub struct MultisigTransaction {
    pub to: [u8; 20],
    pub value: u64,
    pub data: &'static [u8],
    pub nonce: u64,
}

impl MultisigTransaction {
    /// Create a new multisig transaction
    pub fn new(to: [u8; 20], value: u64, data: &'static [u8], nonce: u64) -> Self {
        Self {
            to,
            value,
            data,
            nonce,
        }
    }

    /// Compute the struct hash for this transaction
    pub fn hash_struct(&self) -> [u8; 32] {
        // MultisigTransaction type hash
        let type_hash =
            keccak256(b"MultisigTransaction(address to,uint256 value,bytes data,uint256 nonce)");

        let data_hash = keccak256(self.data);

        // Encode: typeHash || to || value || dataHash || nonce
        let mut encoded = [0u8; 160]; // 32 + 32 + 32 + 32 + 32
        encoded[0..32].copy_from_slice(&type_hash);

        // Encode address (20 bytes, left-padded to 32 bytes)
        encoded[44..64].copy_from_slice(&self.to);

        // Encode value as uint256 (32 bytes, big-endian)
        let value_bytes = self.value.to_be_bytes();
        encoded[88..96].copy_from_slice(&value_bytes);

        // Encode data hash
        encoded[96..128].copy_from_slice(&data_hash);

        // Encode nonce as uint256 (32 bytes, big-endian)
        let nonce_bytes = self.nonce.to_be_bytes();
        encoded[152..160].copy_from_slice(&nonce_bytes);

        keccak256(&encoded)
    }
}

/// Compute EIP712 typed data hash
pub fn eip712_hash(domain: &Eip712Domain, struct_hash: &[u8; 32]) -> [u8; 32] {
    let domain_separator = domain.hash_struct();

    // EIP712 message: "\x19\x01" || domainSeparator || structHash
    let mut message = [0u8; 66];
    message[0] = 0x19;
    message[1] = 0x01;
    message[2..34].copy_from_slice(&domain_separator);
    message[34..66].copy_from_slice(struct_hash);

    keccak256(&message)
}

/// Sign EIP712 typed data
pub fn sign_eip712(
    keypair: &EthereumKeyPair,
    domain: &Eip712Domain,
    struct_hash: &[u8; 32],
) -> Signature {
    let message_hash = eip712_hash(domain, struct_hash);
    keypair.signing_key.sign(&message_hash)
}

/// Verify EIP712 typed data signature
pub fn verify_eip712(
    keypair: &EthereumKeyPair,
    domain: &Eip712Domain,
    struct_hash: &[u8; 32],
    signature: &Signature,
) -> Result<(), ecdsa::Error> {
    let message_hash = eip712_hash(domain, struct_hash);
    keypair.verifying_key.verify(&message_hash, signature)
}
