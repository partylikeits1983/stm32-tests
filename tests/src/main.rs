use p256::ecdsa::{SigningKey, Signature, signature::Signer, VerifyingKey, signature::Verifier};
use sha2::{Sha256, Digest};
use rand_core::OsRng;

fn main() {
    println!("=== ECDSA Signature Test ===\n");
    
    // Generate a signing key
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    
    println!("Generated key pair");
    println!("Public key (hex): {}", hex::encode(verifying_key.to_encoded_point(false).as_bytes()));
    
    // Message to sign
    let message = b"Hello, STM32!";
    println!("\nMessage: {:?}", core::str::from_utf8(message).unwrap());
    
    // Hash the message
    let mut hasher = Sha256::new();
    hasher.update(message);
    let digest = hasher.finalize();
    println!("Message hash (hex): {}", hex::encode(&digest));
    
    // Sign the hash
    let signature: Signature = signing_key.sign(&digest);
    println!("\nSignature (hex): {}", hex::encode(signature.to_bytes()));
    
    // Verify the signature
    match verifying_key.verify(&digest, &signature) {
        Ok(_) => println!("\n✓ Signature verification PASSED"),
        Err(e) => println!("\n✗ Signature verification FAILED: {:?}", e),
    }
    
    // Test with wrong message
    println!("\n--- Testing with wrong message ---");
    let wrong_message = b"Wrong message!";
    let mut hasher = Sha256::new();
    hasher.update(wrong_message);
    let wrong_digest = hasher.finalize();
    
    match verifying_key.verify(&wrong_digest, &signature) {
        Ok(_) => println!("✗ Verification should have failed but passed!"),
        Err(_) => println!("✓ Verification correctly FAILED for wrong message"),
    }
}