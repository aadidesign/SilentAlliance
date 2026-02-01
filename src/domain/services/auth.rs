//! Authentication service
//!
//! Handles JWT token generation, validation, and refresh token rotation.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::JwtSettings;
use crate::errors::ApiError;
use crate::infrastructure::crypto::CryptoService;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (identity ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Not before
    pub nbf: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// JWT ID (for tracking)
    pub jti: String,
    /// Identity fingerprint
    pub fingerprint: String,
    /// Token type
    pub token_type: TokenType,
}

/// Token types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Authentication token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (JWT)
    pub access_token: String,
    /// Refresh token (opaque)
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Access token expiry in seconds
    pub expires_in: i64,
    /// Refresh token expiry in seconds
    pub refresh_expires_in: i64,
}

/// JWT service for token operations
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    settings: JwtSettings,
    validation: Validation,
}

impl JwtService {
    /// Create a new JWT service
    pub fn new(settings: &JwtSettings) -> Result<Self, ApiError> {
        // Parse RSA keys
        let encoding_key = EncodingKey::from_rsa_pem(settings.private_key.as_bytes())
            .map_err(|e| {
                ApiError::CryptoError(format!("Invalid JWT private key: {}", e))
            })?;

        let decoding_key = DecodingKey::from_rsa_pem(settings.public_key.as_bytes())
            .map_err(|e| {
                ApiError::CryptoError(format!("Invalid JWT public key: {}", e))
            })?;

        // Configure validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&settings.issuer]);
        validation.set_audience(&[&settings.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            encoding_key,
            decoding_key,
            settings: settings.clone(),
            validation,
        })
    }

    /// Generate access token
    pub fn generate_access_token(
        &self,
        identity_id: Uuid,
        fingerprint: &str,
    ) -> Result<String, ApiError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.settings.access_token_expiry);

        let claims = Claims {
            sub: identity_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iss: self.settings.issuer.clone(),
            aud: self.settings.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            fingerprint: fingerprint.to_string(),
            token_type: TokenType::Access,
        };

        let header = Header::new(Algorithm::RS256);

        encode(&header, &claims, &self.encoding_key).map_err(|e| {
            ApiError::CryptoError(format!("Failed to generate access token: {}", e))
        })
    }

    /// Generate refresh token (opaque token)
    pub fn generate_refresh_token() -> (String, String) {
        let token = CryptoService::random_token(32);
        let token_hash = CryptoService::sha256_hex(token.as_bytes());
        (token, token_hash)
    }

    /// Generate a token pair
    pub fn generate_token_pair(
        &self,
        identity_id: Uuid,
        fingerprint: &str,
    ) -> Result<(TokenPair, String, Uuid), ApiError> {
        let access_token = self.generate_access_token(identity_id, fingerprint)?;
        let (refresh_token, refresh_token_hash) = Self::generate_refresh_token();
        let family_id = Uuid::new_v4();

        let token_pair = TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.settings.access_token_expiry,
            refresh_expires_in: self.settings.refresh_token_expiry,
        };

        debug!(identity_id = %identity_id, "Token pair generated");

        Ok((token_pair, refresh_token_hash, family_id))
    }

    /// Validate and decode an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, ApiError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;

        // Verify it's an access token
        if token_data.claims.token_type != TokenType::Access {
            return Err(ApiError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    /// Extract claims without validation (for debugging/logging)
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims, ApiError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.set_issuer::<&str>(&[]);
        validation.set_audience::<&str>(&[]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|_| ApiError::InvalidToken)?;

        Ok(token_data.claims)
    }

    /// Get access token expiry duration
    pub fn access_token_expiry(&self) -> chrono::Duration {
        Duration::seconds(self.settings.access_token_expiry)
    }

    /// Get refresh token expiry duration
    pub fn refresh_token_expiry(&self) -> chrono::Duration {
        Duration::seconds(self.settings.refresh_token_expiry)
    }

    /// Hash a refresh token for storage
    pub fn hash_refresh_token(token: &str) -> String {
        CryptoService::sha256_hex(token.as_bytes())
    }
}

/// Challenge-response authentication service
pub struct ChallengeAuthService;

impl ChallengeAuthService {
    /// Generate a new authentication challenge
    pub fn generate_challenge() -> AuthChallenge {
        let challenge_data = CryptoService::generate_challenge();

        AuthChallenge {
            challenge: challenge_data.challenge,
            expires_at: challenge_data.expires_at,
        }
    }

    /// Verify a challenge response
    pub fn verify_response(
        public_key: &[u8],
        challenge: &str,
        signature: &[u8],
    ) -> Result<bool, ApiError> {
        // Check challenge format and extract timestamp
        let parts: Vec<&str> = challenge.split(':').collect();
        if parts.len() != 3 || parts[0] != "silentalliance" {
            return Err(ApiError::InvalidInput("Invalid challenge format".to_string()));
        }

        let timestamp: i64 = parts[1].parse().map_err(|_| {
            ApiError::InvalidInput("Invalid challenge timestamp".to_string())
        })?;

        // Check if challenge has expired (5 minute window)
        let now = Utc::now().timestamp();
        if now - timestamp > 300 {
            return Err(ApiError::TokenExpired);
        }

        // Verify signature
        CryptoService::verify_ed25519_signature(public_key, challenge.as_bytes(), signature)
    }
}

/// Authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge: String,
    pub expires_at: i64,
}

/// OAuth state management
pub struct OAuthStateManager;

impl OAuthStateManager {
    /// Generate OAuth state with HMAC
    pub fn generate_state(crypto: &CryptoService, provider: &str) -> (String, String) {
        let random = CryptoService::random_hex(16);
        let timestamp = Utc::now().timestamp();
        let state_data = format!("{}:{}:{}", provider, timestamp, random);

        let mac = crypto.hmac_sha256(state_data.as_bytes());
        let mac_hex = hex::encode(&mac[..16]); // Use first 16 bytes

        let state = format!("{}.{}", state_data, mac_hex);
        let state_b64 = BASE64.encode(&state);

        (state_b64, random)
    }

    /// Verify OAuth state
    pub fn verify_state(
        crypto: &CryptoService,
        state_b64: &str,
        max_age_seconds: i64,
    ) -> Result<String, ApiError> {
        let state = String::from_utf8(
            BASE64.decode(state_b64).map_err(|_| ApiError::InvalidOAuthState)?
        ).map_err(|_| ApiError::InvalidOAuthState)?;

        let parts: Vec<&str> = state.split('.').collect();
        if parts.len() != 2 {
            return Err(ApiError::InvalidOAuthState);
        }

        let state_data = parts[0];
        let provided_mac = parts[1];

        // Verify HMAC
        let expected_mac = crypto.hmac_sha256(state_data.as_bytes());
        let expected_mac_hex = hex::encode(&expected_mac[..16]);

        if provided_mac != expected_mac_hex {
            warn!("OAuth state HMAC mismatch");
            return Err(ApiError::InvalidOAuthState);
        }

        // Parse and verify timestamp
        let data_parts: Vec<&str> = state_data.split(':').collect();
        if data_parts.len() != 3 {
            return Err(ApiError::InvalidOAuthState);
        }

        let timestamp: i64 = data_parts[1].parse().map_err(|_| ApiError::InvalidOAuthState)?;
        let now = Utc::now().timestamp();

        if now - timestamp > max_age_seconds {
            warn!("OAuth state expired");
            return Err(ApiError::InvalidOAuthState);
        }

        Ok(data_parts[0].to_string()) // Return provider
    }
}

/// PKCE code verifier and challenge
pub struct PkceService;

impl PkceService {
    /// Generate a PKCE code verifier
    pub fn generate_verifier() -> String {
        CryptoService::generate_pkce_verifier()
    }

    /// Generate code challenge from verifier (S256)
    pub fn generate_challenge(verifier: &str) -> String {
        CryptoService::pkce_challenge(verifier)
    }

    /// Verify code verifier against challenge
    pub fn verify(verifier: &str, challenge: &str) -> bool {
        CryptoService::verify_pkce(verifier, challenge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require RSA key generation
    // In production, use environment-based key loading
}
