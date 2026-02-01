//! Authentication request and response types

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Registration request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    /// Ed25519 public key (base64 encoded)
    #[validate(length(min = 32, max = 64, message = "Invalid public key length"))]
    pub public_key: String,
    /// Optional display name
    #[validate(length(min = 1, max = 50, message = "Display name must be 1-50 characters"))]
    pub display_name: Option<String>,
}

/// Registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub identity_id: uuid::Uuid,
    pub fingerprint: String,
    pub challenge: String,
}

/// Challenge request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChallengeRequest {
    /// Public key fingerprint
    #[validate(length(equal = 64, message = "Invalid fingerprint length"))]
    pub fingerprint: String,
}

/// Challenge response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub expires_at: i64,
}

/// Login request with challenge-response
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    /// Public key fingerprint
    #[validate(length(equal = 64, message = "Invalid fingerprint length"))]
    pub fingerprint: String,
    /// The challenge that was signed
    pub challenge: String,
    /// Ed25519 signature of the challenge (base64 encoded)
    pub signature: String,
}

/// Login response with tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub identity: IdentitySummary,
}

/// Identity summary for login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySummary {
    pub id: uuid::Uuid,
    pub fingerprint: String,
    pub display_name: Option<String>,
    pub karma: i32,
}

/// Token refresh request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Token refresh response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Logout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// OAuth authorization request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OAuthAuthorizeRequest {
    /// OAuth provider (github, discord)
    pub provider: String,
    /// PKCE code challenge
    pub code_challenge: String,
    /// Challenge method (always S256)
    #[serde(default = "default_challenge_method")]
    pub code_challenge_method: String,
    /// Redirect URI for the callback
    pub redirect_uri: Option<String>,
    /// State for CSRF protection
    pub state: Option<String>,
}

fn default_challenge_method() -> String {
    "S256".to_string()
}

/// OAuth authorization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
}

/// OAuth callback parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

/// OAuth token exchange request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OAuthTokenRequest {
    pub grant_type: String,
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

/// OAuth token exchange response (same as login)
pub type OAuthTokenResponse = LoginResponse;

/// OAuth user info (from provider)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}
