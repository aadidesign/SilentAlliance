//! Authentication middleware
//!
//! Provides JWT token verification and user extraction.

use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::domain::services::auth::JwtService;
use crate::errors::ApiError;
use crate::AppState;

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// Identity ID from the token
    pub identity_id: Uuid,
    /// Public key fingerprint
    pub fingerprint: String,
    /// JWT ID for tracking
    pub jti: String,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Get Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // Extract Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        // Validate token
        let jwt_service = JwtService::new(&state.settings.jwt)
            .map_err(|_| ApiError::InternalError)?;

        let claims = jwt_service.validate_access_token(token)?;

        // Parse identity ID
        let identity_id = claims.sub.parse::<Uuid>()
            .map_err(|_| ApiError::InvalidToken)?;

        // Optionally check if user is suspended (could be cached)
        let is_suspended: Option<bool> = sqlx::query_scalar!(
            "SELECT is_suspended FROM identities WHERE id = $1",
            identity_id
        )
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| ApiError::InternalError)?
        .flatten();

        if is_suspended == Some(true) {
            return Err(ApiError::AccountSuspended("Account is suspended".to_string()));
        }

        debug!(identity_id = %identity_id, "User authenticated");

        Ok(AuthenticatedUser {
            identity_id,
            fingerprint: claims.fingerprint,
            jti: claims.jti,
        })
    }
}

/// Optional authentication - extracts user if token is present
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthenticatedUser>);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for OptionalUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalUser(Some(user))),
            Err(ApiError::Unauthorized) => Ok(OptionalUser(None)),
            Err(e) => Err(e),
        }
    }
}

/// Authentication middleware for routes that require auth
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token
    let jwt_service = match JwtService::new(&state.settings.jwt) {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let claims = match jwt_service.validate_access_token(token) {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "Token validation failed");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Store claims in request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Require moderator or admin role
#[derive(Debug, Clone)]
pub struct ModeratorUser {
    pub user: AuthenticatedUser,
    pub space_id: Option<Uuid>,
}

/// Check if user is a moderator of a space
pub async fn check_moderator(
    state: &Arc<AppState>,
    identity_id: Uuid,
    space_id: Uuid,
) -> Result<bool, ApiError> {
    let is_mod: Option<bool> = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM space_members
            WHERE space_id = $1 AND identity_id = $2 AND role IN ('moderator', 'admin')
        ) as "exists!"
        "#,
        space_id,
        identity_id
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(is_mod.unwrap_or(false))
}

/// Check if user is the creator/admin of a space
pub async fn check_space_admin(
    state: &Arc<AppState>,
    identity_id: Uuid,
    space_id: Uuid,
) -> Result<bool, ApiError> {
    let is_admin: Option<bool> = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM space_members
            WHERE space_id = $1 AND identity_id = $2 AND role = 'admin'
        ) OR EXISTS(
            SELECT 1 FROM spaces
            WHERE id = $1 AND creator_id = $2
        ) as "exists!"
        "#,
        space_id,
        identity_id
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(is_admin.unwrap_or(false))
}
