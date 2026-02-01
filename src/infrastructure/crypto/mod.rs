//! Cryptographic infrastructure module
//!
//! Provides cryptographic operations including:
//! - Ed25519 signature verification
//! - X25519 key exchange for E2E encryption
//! - ChaCha20-Poly1305 symmetric encryption
//! - Argon2id password hashing
//! - SHA-256 hashing
//! - HMAC-SHA256 for message authentication
//! - Secure random generation

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use tracing::{debug, error};

use crate::config::CryptoSettings;
use crate::errors::ApiError;

type HmacSha256 = Hmac<Sha256>;

/// Cryptographic service providing all crypto operations
#[derive(Clone)]
pub struct CryptoService {
    /// Argon2 hasher with configured parameters
    argon2: Argon2<'static>,
    /// Master key for HMAC operations
    master_key: Vec<u8>,
}

impl CryptoService {
    /// Create a new crypto service with the given settings
    pub fn new(settings: &CryptoSettings) -> Result<Self, ApiError> {
        // Decode master key from base64
        let master_key = BASE64.decode(&settings.master_key).map_err(|e| {
            error!(error = %e, "Failed to decode master key");
            ApiError::CryptoError("Invalid master key encoding".to_string())
        })?;

        if master_key.len() != 32 {
            return Err(ApiError::CryptoError(
                "Master key must be 32 bytes".to_string(),
            ));
        }

        // Configure Argon2id with custom parameters
        let params = Params::new(
            settings.argon2_memory_cost,
            settings.argon2_time_cost,
            settings.argon2_parallelism,
            Some(32), // Output length
        )
        .map_err(|e| {
            error!(error = %e, "Failed to create Argon2 params");
            ApiError::CryptoError("Invalid Argon2 parameters".to_string())
        })?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Ok(Self { argon2, master_key })
    }

    // ==================== Password Hashing ====================

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                error!(error = %e, "Failed to hash password");
                ApiError::CryptoError("Password hashing failed".to_string())
            })?
            .to_string();

        debug!("Password hashed successfully");
        Ok(hash)
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApiError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            error!(error = %e, "Failed to parse password hash");
            ApiError::CryptoError("Invalid password hash format".to_string())
        })?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => {
                error!(error = %e, "Password verification error");
                Err(ApiError::CryptoError("Password verification failed".to_string()))
            }
        }
    }

    // ==================== Ed25519 Signatures ====================

    /// Generate a new Ed25519 keypair
    /// Returns (private_key, public_key) as base64 encoded strings
    pub fn generate_ed25519_keypair() -> (String, String) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let private_key = BASE64.encode(signing_key.to_bytes());
        let public_key = BASE64.encode(verifying_key.to_bytes());

        (private_key, public_key)
    }

    /// Verify an Ed25519 signature
    pub fn verify_ed25519_signature(
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, ApiError> {
        // Parse public key (32 bytes)
        let verifying_key_bytes: [u8; 32] = public_key.try_into().map_err(|_| {
            ApiError::InvalidPublicKey
        })?;

        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes)
            .map_err(|_| ApiError::InvalidPublicKey)?;

        // Parse signature (64 bytes)
        let signature_bytes: [u8; 64] = signature.try_into().map_err(|_| {
            ApiError::InvalidSignature
        })?;

        let signature = Signature::from_bytes(&signature_bytes);

        match verifying_key.verify(message, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Sign a message with an Ed25519 private key
    pub fn sign_ed25519(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>, ApiError> {
        let signing_key_bytes: [u8; 32] = private_key.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid private key length".to_string())
        })?;

        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let signature = signing_key.sign(message);

        Ok(signature.to_bytes().to_vec())
    }

    /// Calculate Ed25519 public key fingerprint (SHA-256 hash, hex encoded)
    pub fn public_key_fingerprint(public_key: &[u8]) -> String {
        let hash = Sha256::digest(public_key);
        hex::encode(hash)
    }

    // ==================== X25519 Key Exchange ====================

    /// Generate a new X25519 keypair for key exchange
    /// Returns (private_key, public_key) as base64 encoded strings
    pub fn generate_x25519_keypair() -> (String, String) {
        let private_key = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = X25519PublicKey::from(&private_key);

        let private_key_b64 = BASE64.encode(private_key.as_bytes());
        let public_key_b64 = BASE64.encode(public_key.as_bytes());

        (private_key_b64, public_key_b64)
    }

    /// Perform X25519 key exchange
    /// Returns the shared secret
    pub fn x25519_key_exchange(
        our_private_key: &[u8],
        their_public_key: &[u8],
    ) -> Result<Vec<u8>, ApiError> {
        let private_key_bytes: [u8; 32] = our_private_key.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid private key length".to_string())
        })?;

        let public_key_bytes: [u8; 32] = their_public_key.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid public key length".to_string())
        })?;

        let private_key = StaticSecret::from(private_key_bytes);
        let public_key = X25519PublicKey::from(public_key_bytes);

        let shared_secret = private_key.diffie_hellman(&public_key);

        Ok(shared_secret.as_bytes().to_vec())
    }

    // ==================== ChaCha20-Poly1305 Encryption ====================

    /// Encrypt data using ChaCha20-Poly1305
    /// Returns (ciphertext, nonce)
    pub fn encrypt_chacha20(
        key: &[u8],
        plaintext: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>), ApiError> {
        let key_bytes: [u8; 32] = key.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid key length for ChaCha20".to_string())
        })?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|_| ApiError::CryptoError("Failed to create cipher".to_string()))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = match aad {
            Some(aad_data) => {
                let payload = Payload {
                    msg: plaintext,
                    aad: aad_data,
                };
                cipher.encrypt(nonce, payload)
            }
            None => cipher.encrypt(nonce, plaintext),
        }
        .map_err(|e| {
            error!(error = %e, "Encryption failed");
            ApiError::CryptoError("Encryption failed".to_string())
        })?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// Decrypt data using ChaCha20-Poly1305
    pub fn decrypt_chacha20(
        key: &[u8],
        ciphertext: &[u8],
        nonce: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>, ApiError> {
        let key_bytes: [u8; 32] = key.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid key length for ChaCha20".to_string())
        })?;

        let nonce_bytes: [u8; 12] = nonce.try_into().map_err(|_| {
            ApiError::CryptoError("Invalid nonce length".to_string())
        })?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|_| ApiError::CryptoError("Failed to create cipher".to_string()))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = match aad {
            Some(aad_data) => {
                let payload = Payload {
                    msg: ciphertext,
                    aad: aad_data,
                };
                cipher.decrypt(nonce, payload)
            }
            None => cipher.decrypt(nonce, ciphertext),
        }
        .map_err(|e| {
            error!(error = %e, "Decryption failed");
            ApiError::CryptoError("Decryption failed - invalid ciphertext or key".to_string())
        })?;

        Ok(plaintext)
    }

    // ==================== Hashing & HMAC ====================

    /// Calculate SHA-256 hash
    pub fn sha256(data: &[u8]) -> Vec<u8> {
        Sha256::digest(data).to_vec()
    }

    /// Calculate SHA-256 hash and return as hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }

    /// Calculate HMAC-SHA256
    pub fn hmac_sha256(&self, data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(&self.master_key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify HMAC-SHA256
    pub fn verify_hmac_sha256(&self, data: &[u8], expected_mac: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(&self.master_key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.verify_slice(expected_mac).is_ok()
    }

    /// Calculate HMAC-SHA256 with custom key
    pub fn hmac_sha256_with_key(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    // ==================== Random Generation ====================

    /// Generate cryptographically secure random bytes
    pub fn random_bytes(len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Generate a random hex string
    pub fn random_hex(len: usize) -> String {
        hex::encode(Self::random_bytes(len))
    }

    /// Generate a random base64 string
    pub fn random_base64(len: usize) -> String {
        BASE64.encode(Self::random_bytes(len))
    }

    /// Generate a random token (URL-safe base64)
    pub fn random_token(len: usize) -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        URL_SAFE_NO_PAD.encode(Self::random_bytes(len))
    }

    // ==================== Challenge-Response ====================

    /// Generate a challenge for signature verification
    pub fn generate_challenge() -> ChallengeData {
        let nonce = Self::random_hex(32);
        let timestamp = chrono::Utc::now().timestamp();
        let challenge = format!("silentalliance:{}:{}", timestamp, nonce);

        ChallengeData {
            challenge,
            nonce,
            timestamp,
            expires_at: timestamp + 300, // 5 minute expiry
        }
    }

    /// Verify a challenge response
    pub fn verify_challenge_response(
        challenge: &str,
        public_key: &[u8],
        signature: &[u8],
        max_age_seconds: i64,
    ) -> Result<bool, ApiError> {
        // Parse challenge to check timestamp
        let parts: Vec<&str> = challenge.split(':').collect();
        if parts.len() != 3 || parts[0] != "silentalliance" {
            return Err(ApiError::InvalidInput("Invalid challenge format".to_string()));
        }

        let timestamp: i64 = parts[1].parse().map_err(|_| {
            ApiError::InvalidInput("Invalid challenge timestamp".to_string())
        })?;

        let now = chrono::Utc::now().timestamp();
        if now - timestamp > max_age_seconds {
            return Err(ApiError::TokenExpired);
        }

        // Verify signature
        Self::verify_ed25519_signature(public_key, challenge.as_bytes(), signature)
    }

    // ==================== PKCE ====================

    /// Generate a PKCE code verifier
    pub fn generate_pkce_verifier() -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        URL_SAFE_NO_PAD.encode(Self::random_bytes(32))
    }

    /// Calculate PKCE code challenge from verifier (S256 method)
    pub fn pkce_challenge(verifier: &str) -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let hash = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Verify PKCE challenge against verifier
    pub fn verify_pkce(verifier: &str, challenge: &str) -> bool {
        Self::pkce_challenge(verifier) == challenge
    }
}

/// Challenge data for authentication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeData {
    pub challenge: String,
    pub nonce: String,
    pub timestamp: i64,
    pub expires_at: i64,
}

/// Encrypted message structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedMessage {
    pub ciphertext: String, // Base64 encoded
    pub nonce: String,      // Base64 encoded
}

impl EncryptedMessage {
    /// Create from raw bytes
    pub fn from_bytes(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce),
        }
    }

    /// Get ciphertext bytes
    pub fn ciphertext_bytes(&self) -> Result<Vec<u8>, ApiError> {
        BASE64.decode(&self.ciphertext).map_err(|_| {
            ApiError::CryptoError("Invalid ciphertext encoding".to_string())
        })
    }

    /// Get nonce bytes
    pub fn nonce_bytes(&self) -> Result<Vec<u8>, ApiError> {
        BASE64.decode(&self.nonce).map_err(|_| {
            ApiError::CryptoError("Invalid nonce encoding".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_crypto_service() -> CryptoService {
        let settings = CryptoSettings {
            master_key: BASE64.encode(&[0u8; 32]),
            argon2_memory_cost: 4096,
            argon2_time_cost: 1,
            argon2_parallelism: 1,
        };
        CryptoService::new(&settings).unwrap()
    }

    #[test]
    fn test_password_hashing() {
        let crypto = test_crypto_service();
        let password = "secure_password_123!";

        let hash = crypto.hash_password(password).unwrap();
        assert!(crypto.verify_password(password, &hash).unwrap());
        assert!(!crypto.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_ed25519_keypair_and_signature() {
        let (private_key_b64, public_key_b64) = CryptoService::generate_ed25519_keypair();

        let private_key = BASE64.decode(&private_key_b64).unwrap();
        let public_key = BASE64.decode(&public_key_b64).unwrap();

        let message = b"Hello, World!";
        let signature = CryptoService::sign_ed25519(&private_key, message).unwrap();

        assert!(CryptoService::verify_ed25519_signature(&public_key, message, &signature).unwrap());
        assert!(!CryptoService::verify_ed25519_signature(&public_key, b"Wrong message", &signature).unwrap());
    }

    #[test]
    fn test_x25519_key_exchange() {
        let (alice_private, alice_public) = CryptoService::generate_x25519_keypair();
        let (bob_private, bob_public) = CryptoService::generate_x25519_keypair();

        let alice_private_bytes = BASE64.decode(&alice_private).unwrap();
        let bob_public_bytes = BASE64.decode(&bob_public).unwrap();

        let bob_private_bytes = BASE64.decode(&bob_private).unwrap();
        let alice_public_bytes = BASE64.decode(&alice_public).unwrap();

        let alice_shared = CryptoService::x25519_key_exchange(&alice_private_bytes, &bob_public_bytes).unwrap();
        let bob_shared = CryptoService::x25519_key_exchange(&bob_private_bytes, &alice_public_bytes).unwrap();

        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_chacha20_encryption() {
        let key = CryptoService::random_bytes(32);
        let plaintext = b"Secret message for testing encryption";

        let (ciphertext, nonce) = CryptoService::encrypt_chacha20(&key, plaintext, None).unwrap();
        let decrypted = CryptoService::decrypt_chacha20(&key, &ciphertext, &nonce, None).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_chacha20_with_aad() {
        let key = CryptoService::random_bytes(32);
        let plaintext = b"Secret message";
        let aad = b"additional authenticated data";

        let (ciphertext, nonce) = CryptoService::encrypt_chacha20(&key, plaintext, Some(aad)).unwrap();
        let decrypted = CryptoService::decrypt_chacha20(&key, &ciphertext, &nonce, Some(aad)).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);

        // Should fail with wrong AAD
        let result = CryptoService::decrypt_chacha20(&key, &ciphertext, &nonce, Some(b"wrong aad"));
        assert!(result.is_err());
    }

    #[test]
    fn test_sha256() {
        let data = b"Hello, World!";
        let hash = CryptoService::sha256_hex(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_hmac() {
        let crypto = test_crypto_service();
        let data = b"Message to authenticate";

        let mac = crypto.hmac_sha256(data);
        assert!(crypto.verify_hmac_sha256(data, &mac));
        assert!(!crypto.verify_hmac_sha256(b"Wrong data", &mac));
    }

    #[test]
    fn test_pkce() {
        let verifier = CryptoService::generate_pkce_verifier();
        let challenge = CryptoService::pkce_challenge(&verifier);

        assert!(CryptoService::verify_pkce(&verifier, &challenge));
        assert!(!CryptoService::verify_pkce("wrong_verifier", &challenge));
    }

    #[test]
    fn test_random_generation() {
        let bytes1 = CryptoService::random_bytes(32);
        let bytes2 = CryptoService::random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different (extremely unlikely to be equal)
    }

    #[test]
    fn test_public_key_fingerprint() {
        let (_, public_key_b64) = CryptoService::generate_ed25519_keypair();
        let public_key = BASE64.decode(&public_key_b64).unwrap();

        let fingerprint = CryptoService::public_key_fingerprint(&public_key);
        assert_eq!(fingerprint.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}
