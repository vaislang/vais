//! Ed25519 signature verification for package publishing
//!
//! Packages can optionally be signed with Ed25519 keys. When a user
//! registers a public key, all subsequent publishes must include a valid
//! signature over the archive checksum.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Errors that may occur during signature operations
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("signature verification failed")]
    VerificationFailed,

    #[error("signature required but not provided")]
    SignatureRequired,

    #[error("hex decode error: {0}")]
    HexDecode(String),
}

/// Verify an Ed25519 signature over a SHA-256 digest of the archive data.
///
/// The signing scheme:
/// 1. Compute SHA-256(archive_bytes) to get the 32-byte digest
/// 2. The publisher signs the digest with their Ed25519 private key
/// 3. We verify the signature using the publisher's registered public key
pub fn verify_signature(
    public_key_hex: &str,
    archive_data: &[u8],
    signature_hex: &str,
) -> Result<(), SignatureError> {
    // Decode public key from hex (32 bytes)
    let pk_bytes =
        hex::decode(public_key_hex).map_err(|e| SignatureError::HexDecode(e.to_string()))?;

    if pk_bytes.len() != 32 {
        return Err(SignatureError::InvalidPublicKey(format!(
            "expected 32 bytes, got {}",
            pk_bytes.len()
        )));
    }

    let pk_array: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|_| SignatureError::InvalidPublicKey("invalid key length".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

    // Decode signature from hex (64 bytes)
    let sig_bytes =
        hex::decode(signature_hex).map_err(|e| SignatureError::HexDecode(e.to_string()))?;

    if sig_bytes.len() != 64 {
        return Err(SignatureError::InvalidSignature(format!(
            "expected 64 bytes, got {}",
            sig_bytes.len()
        )));
    }

    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| SignatureError::InvalidSignature("invalid signature length".to_string()))?;

    let signature = Signature::from_bytes(&sig_array);

    // Compute SHA-256 digest of archive data
    let mut hasher = Sha256::new();
    hasher.update(archive_data);
    let digest = hasher.finalize();

    // Verify signature over the digest
    verifying_key
        .verify(&digest, &signature)
        .map_err(|_| SignatureError::VerificationFailed)
}

/// Validate that a hex-encoded public key is well-formed
pub fn validate_public_key(public_key_hex: &str) -> Result<(), SignatureError> {
    let pk_bytes =
        hex::decode(public_key_hex).map_err(|e| SignatureError::HexDecode(e.to_string()))?;

    if pk_bytes.len() != 32 {
        return Err(SignatureError::InvalidPublicKey(format!(
            "expected 32 bytes, got {}",
            pk_bytes.len()
        )));
    }

    let pk_array: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|_| SignatureError::InvalidPublicKey("invalid key length".to_string()))?;

    VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn sign_archive(signing_key: &SigningKey, archive_data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(archive_data);
        let digest = hasher.finalize();
        let sig = signing_key.sign(&digest);
        hex::encode(sig.to_bytes())
    }

    #[test]
    fn test_verify_valid_signature() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let pk_hex = hex::encode(verifying_key.as_bytes());

        let archive_data = b"hello world package data";
        let sig_hex = sign_archive(&signing_key, archive_data);

        let result = verify_signature(&pk_hex, archive_data, &sig_hex);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let pk_hex = hex::encode(verifying_key.as_bytes());

        let archive_data = b"hello world package data";
        let sig_hex = sign_archive(&signing_key, archive_data);

        // Tamper with the data
        let tampered = b"tampered package data";
        let result = verify_signature(&pk_hex, tampered, &sig_hex);
        assert!(matches!(result, Err(SignatureError::VerificationFailed)));
    }

    #[test]
    fn test_verify_wrong_key() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let other_key = SigningKey::generate(&mut OsRng);
        let other_pk_hex = hex::encode(other_key.verifying_key().as_bytes());

        let archive_data = b"hello world package data";
        let sig_hex = sign_archive(&signing_key, archive_data);

        let result = verify_signature(&other_pk_hex, archive_data, &sig_hex);
        assert!(matches!(result, Err(SignatureError::VerificationFailed)));
    }

    #[test]
    fn test_invalid_public_key_hex() {
        let result = verify_signature("not_valid_hex!", b"data", &hex::encode([0u8; 64]));
        assert!(matches!(result, Err(SignatureError::HexDecode(_))));
    }

    #[test]
    fn test_invalid_public_key_length() {
        let short_key = hex::encode([0u8; 16]);
        let result = verify_signature(&short_key, b"data", &hex::encode([0u8; 64]));
        assert!(matches!(result, Err(SignatureError::InvalidPublicKey(_))));
    }

    #[test]
    fn test_invalid_signature_hex() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());

        let result = verify_signature(&pk_hex, b"data", "not_valid_hex!");
        assert!(matches!(result, Err(SignatureError::HexDecode(_))));
    }

    #[test]
    fn test_invalid_signature_length() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let short_sig = hex::encode([0u8; 32]);

        let result = verify_signature(&pk_hex, b"data", &short_sig);
        assert!(matches!(result, Err(SignatureError::InvalidSignature(_))));
    }

    #[test]
    fn test_validate_public_key_valid() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        assert!(validate_public_key(&pk_hex).is_ok());
    }

    #[test]
    fn test_validate_public_key_invalid_hex() {
        assert!(validate_public_key("not_hex").is_err());
    }

    #[test]
    fn test_validate_public_key_wrong_length() {
        let short = hex::encode([0u8; 16]);
        assert!(validate_public_key(&short).is_err());
    }

    #[test]
    fn test_deterministic_signatures() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let data = b"same data";

        let sig1 = sign_archive(&signing_key, data);
        let sig2 = sign_archive(&signing_key, data);

        // Ed25519 signatures are deterministic (RFC 8032)
        assert_eq!(sig1, sig2);

        // Both should verify
        assert!(verify_signature(&pk_hex, data, &sig1).is_ok());
        assert!(verify_signature(&pk_hex, data, &sig2).is_ok());
    }

    #[test]
    fn test_empty_archive() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let data = b"";
        let sig_hex = sign_archive(&signing_key, data);

        assert!(verify_signature(&pk_hex, data, &sig_hex).is_ok());
    }

    #[test]
    fn test_large_archive() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let data = vec![0xABu8; 1_000_000]; // 1MB
        let sig_hex = sign_archive(&signing_key, &data);

        assert!(verify_signature(&pk_hex, &data, &sig_hex).is_ok());
    }

    #[test]
    fn test_error_display() {
        let err = SignatureError::VerificationFailed;
        assert_eq!(err.to_string(), "signature verification failed");

        let err = SignatureError::SignatureRequired;
        assert_eq!(err.to_string(), "signature required but not provided");

        let err = SignatureError::InvalidPublicKey("bad key".to_string());
        assert!(err.to_string().contains("bad key"));

        let err = SignatureError::InvalidSignature("bad sig".to_string());
        assert!(err.to_string().contains("bad sig"));

        let err = SignatureError::HexDecode("invalid".to_string());
        assert!(err.to_string().contains("invalid"));
    }
}
