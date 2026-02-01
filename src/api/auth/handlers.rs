//! Authentication handlers
//!
//! Implements registration, login, OAuth, and token management endpoints.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::domain::services::auth::{
    AuthChallenge, ChallengeAuthService, JwtService, OAuthStateManager, PkceService,
};
use crate::errors::{ApiError, ApiResult};
use crate::infrastructure::crypto::CryptoService;
use crate::AppState;

use super::types::*;

/// Register a new identity with an Ed25519 public key
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<Json<RegisterResponse>> {
    request.validate()?;

    // Decode and validate the public key
    let public_key = BASE64.decode(&request.public_key)
        .map_err(|_| ApiError::InvalidPublicKey)?;

    if public_key.len() != 32 {
        return Err(ApiError::InvalidPublicKey);
    }

    // Calculate fingerprint
    let fingerprint = CryptoService::public_key_fingerprint(&public_key);

    // Check if identity already exists
    let existing = sqlx::query_scalar!(
        r#"SELECT id FROM identities WHERE public_key_fingerprint = $1"#,
        &fingerprint
    )
    .fetch_optional(state.db.pool())
    .await?;

    if existing.is_some() {
        return Err(ApiError::Conflict("Identity already exists".to_string()));
    }

    // Create the identity
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query!(
        r#"
        INSERT INTO identities (id, public_key, public_key_fingerprint, display_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
        id,
        &public_key,
        &fingerprint,
        request.display_name,
        now
    )
    .execute(state.db.pool())
    .await?;

    // Generate a challenge for immediate login
    let challenge = ChallengeAuthService::generate_challenge();

    // Store challenge in Redis
    state.redis.set(
        &format!("challenge:{}", fingerprint),
        &challenge,
        Some(std::time::Duration::from_secs(300)),
    ).await?;

    info!(identity_id = %id, fingerprint = %fingerprint, "New identity registered");

    Ok(Json(RegisterResponse {
        identity_id: id,
        fingerprint,
        challenge: challenge.challenge,
    }))
}

/// Get a challenge for authentication
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChallengeRequest>,
) -> ApiResult<Json<ChallengeResponse>> {
    request.validate()?;

    // Verify identity exists
    let exists = sqlx::query_scalar!(
        r#"SELECT id FROM identities WHERE public_key_fingerprint = $1 AND is_suspended = false"#,
        &request.fingerprint
    )
    .fetch_optional(state.db.pool())
    .await?;

    if exists.is_none() {
        // Don't reveal whether identity exists
        return Err(ApiError::InvalidCredentials);
    }

    // Generate challenge
    let challenge = ChallengeAuthService::generate_challenge();

    // Store in Redis
    state.redis.set(
        &format!("challenge:{}", request.fingerprint),
        &challenge,
        Some(std::time::Duration::from_secs(300)),
    ).await?;

    Ok(Json(ChallengeResponse {
        challenge: challenge.challenge,
        expires_at: challenge.expires_at,
    }))
}

/// Login with challenge-response signature
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    request.validate()?;

    // Get the stored challenge
    let challenge: Option<AuthChallenge> = state.redis
        .get(&format!("challenge:{}", request.fingerprint))
        .await?;

    let challenge = challenge.ok_or(ApiError::InvalidCredentials)?;

    // Verify the challenge matches
    if challenge.challenge != request.challenge {
        return Err(ApiError::InvalidCredentials);
    }

    // Check expiration
    if Utc::now().timestamp() > challenge.expires_at {
        return Err(ApiError::TokenExpired);
    }

    // Get identity and public key
    let identity = sqlx::query!(
        r#"
        SELECT id, public_key, public_key_fingerprint, display_name, karma, is_suspended
        FROM identities
        WHERE public_key_fingerprint = $1
        "#,
        &request.fingerprint
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    if identity.is_suspended {
        return Err(ApiError::AccountSuspended("Account is suspended".to_string()));
    }

    // Verify signature
    let signature = BASE64.decode(&request.signature)
        .map_err(|_| ApiError::InvalidSignature)?;

    let is_valid = CryptoService::verify_ed25519_signature(
        &identity.public_key,
        request.challenge.as_bytes(),
        &signature,
    )?;

    if !is_valid {
        warn!(fingerprint = %request.fingerprint, "Invalid signature during login");
        return Err(ApiError::InvalidCredentials);
    }

    // Delete the used challenge
    state.redis.delete(&format!("challenge:{}", request.fingerprint)).await?;

    // Generate tokens
    let jwt_service = JwtService::new(&state.settings.jwt)
        .map_err(|e| ApiError::CryptoError(e.to_string()))?;

    let (token_pair, refresh_hash, family_id) = jwt_service.generate_token_pair(
        identity.id,
        &identity.public_key_fingerprint,
    )?;

    // Store refresh token
    let refresh_expires = Utc::now() + Duration::seconds(state.settings.jwt.refresh_token_expiry);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (id, identity_id, token_hash, family_id, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        Uuid::new_v4(),
        identity.id,
        &refresh_hash,
        family_id,
        refresh_expires,
        Utc::now()
    )
    .execute(state.db.pool())
    .await?;

    info!(identity_id = %identity.id, "User logged in successfully");

    Ok(Json(LoginResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        identity: IdentitySummary {
            id: identity.id,
            fingerprint: identity.public_key_fingerprint,
            display_name: identity.display_name,
            karma: identity.karma,
        },
    }))
}

/// Refresh an access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshRequest>,
) -> ApiResult<Json<RefreshResponse>> {
    // Hash the provided refresh token
    let token_hash = JwtService::hash_refresh_token(&request.refresh_token);

    // Find the refresh token
    let stored_token = sqlx::query!(
        r#"
        SELECT rt.id, rt.identity_id, rt.family_id, rt.revoked, rt.expires_at,
               i.public_key_fingerprint, i.is_suspended
        FROM refresh_tokens rt
        JOIN identities i ON i.id = rt.identity_id
        WHERE rt.token_hash = $1
        "#,
        &token_hash
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::InvalidToken)?;

    // Check if token is revoked
    if stored_token.revoked {
        // Token reuse detected! Revoke all tokens in this family
        warn!(
            family_id = %stored_token.family_id,
            "Refresh token reuse detected - revoking family"
        );

        sqlx::query!(
            "UPDATE refresh_tokens SET revoked = true WHERE family_id = $1",
            stored_token.family_id
        )
        .execute(state.db.pool())
        .await?;

        return Err(ApiError::RefreshTokenReuse);
    }

    // Check if token is expired
    if Utc::now() > stored_token.expires_at {
        return Err(ApiError::TokenExpired);
    }

    // Check if user is suspended
    if stored_token.is_suspended {
        return Err(ApiError::AccountSuspended("Account is suspended".to_string()));
    }

    // Revoke the old token (single-use)
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = true WHERE id = $1",
        stored_token.id
    )
    .execute(state.db.pool())
    .await?;

    // Generate new tokens (same family for tracking)
    let jwt_service = JwtService::new(&state.settings.jwt)
        .map_err(|e| ApiError::CryptoError(e.to_string()))?;

    let access_token = jwt_service.generate_access_token(
        stored_token.identity_id,
        &stored_token.public_key_fingerprint,
    )?;

    let (new_refresh_token, new_refresh_hash) = JwtService::generate_refresh_token();

    // Store new refresh token in the same family
    let refresh_expires = Utc::now() + Duration::seconds(state.settings.jwt.refresh_token_expiry);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (id, identity_id, token_hash, family_id, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        Uuid::new_v4(),
        stored_token.identity_id,
        &new_refresh_hash,
        stored_token.family_id, // Same family
        refresh_expires,
        Utc::now()
    )
    .execute(state.db.pool())
    .await?;

    debug!(identity_id = %stored_token.identity_id, "Token refreshed");

    Ok(Json(RefreshResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.settings.jwt.access_token_expiry,
    }))
}

/// Logout (revoke refresh token)
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LogoutRequest>,
) -> ApiResult<StatusCode> {
    let token_hash = JwtService::hash_refresh_token(&request.refresh_token);

    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1",
        &token_hash
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Logout from all sessions (revoke all refresh tokens)
pub async fn logout_all(
    State(state): State<Arc<AppState>>,
    // This would normally require authentication
    Json(request): Json<LogoutRequest>,
) -> ApiResult<StatusCode> {
    let token_hash = JwtService::hash_refresh_token(&request.refresh_token);

    // Get the identity ID from the refresh token
    let token = sqlx::query!(
        "SELECT identity_id FROM refresh_tokens WHERE token_hash = $1 AND revoked = false",
        &token_hash
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::InvalidToken)?;

    // Revoke all tokens for this identity
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = true WHERE identity_id = $1",
        token.identity_id
    )
    .execute(state.db.pool())
    .await?;

    info!(identity_id = %token.identity_id, "All sessions revoked");

    Ok(StatusCode::NO_CONTENT)
}

/// Start OAuth authorization flow
pub async fn oauth_authorize(
    State(state): State<Arc<AppState>>,
    Query(request): Query<OAuthAuthorizeRequest>,
) -> ApiResult<Json<OAuthAuthorizeResponse>> {
    // Validate provider
    let provider_config = match request.provider.as_str() {
        "github" => state.settings.oauth.github.as_ref(),
        "discord" => state.settings.oauth.discord.as_ref(),
        _ => return Err(ApiError::InvalidInput(format!("Unknown OAuth provider: {}", request.provider))),
    };

    let provider_config = provider_config
        .ok_or_else(|| ApiError::InvalidInput(format!("OAuth provider {} not configured", request.provider)))?;

    // Generate state with HMAC protection
    let (state_b64, _) = OAuthStateManager::generate_state(&state.crypto, &request.provider);

    // Store PKCE code challenge in Redis
    state.redis.set(
        &format!("oauth:pkce:{}", &state_b64[..32]),
        &request.code_challenge,
        Some(std::time::Duration::from_secs(600)),
    ).await?;

    // Build authorization URL
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&scope={}&state={}&response_type=code",
        provider_config.auth_url,
        urlencoding::encode(&provider_config.client_id),
        urlencoding::encode(&request.redirect_uri.as_deref().unwrap_or(&provider_config.redirect_uri)),
        urlencoding::encode(&provider_config.scopes.join(" ")),
        urlencoding::encode(&state_b64),
    );

    Ok(Json(OAuthAuthorizeResponse {
        authorization_url: auth_url,
        state: state_b64,
    }))
}

/// Handle OAuth callback from provider
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify state
    let verified_provider = OAuthStateManager::verify_state(
        &state.crypto,
        &params.state,
        600, // 10 minute expiry
    )?;

    if verified_provider != provider {
        return Err(ApiError::InvalidOAuthState);
    }

    // Store the authorization code temporarily
    state.redis.set(
        &format!("oauth:code:{}", &params.state[..32]),
        &params.code,
        Some(std::time::Duration::from_secs(300)),
    ).await?;

    // Return a page/JSON indicating to exchange the code
    Ok(Json(serde_json::json!({
        "status": "callback_received",
        "message": "Use the /auth/oauth/token endpoint to exchange the code for tokens",
        "state": params.state
    })))
}

/// Exchange OAuth authorization code for tokens
pub async fn oauth_token_exchange(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OAuthTokenRequest>,
) -> ApiResult<Json<OAuthTokenResponse>> {
    if request.grant_type != "authorization_code" {
        return Err(ApiError::InvalidInput("Invalid grant_type".to_string()));
    }

    // This is a simplified implementation
    // In production, you would:
    // 1. Exchange the code with the OAuth provider for their tokens
    // 2. Fetch user info from the provider
    // 3. Create or link the identity
    // 4. Generate your own tokens

    Err(ApiError::custom(
        StatusCode::NOT_IMPLEMENTED,
        "NOT_IMPLEMENTED",
        "OAuth token exchange not fully implemented in this example",
    ))
}
