//! API Error types and response handling
//!
//! This module defines the unified error type used throughout the API layer.
//! All errors are automatically converted to JSON responses with appropriate HTTP status codes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Unified API error type
#[derive(Debug, Error)]
pub enum ApiError {
    // Authentication & Authorization Errors
    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Refresh token reuse detected - all sessions revoked")]
    RefreshTokenReuse,

    #[error("Access denied")]
    Forbidden,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    // Validation Errors
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key format")]
    InvalidPublicKey,

    // Resource Errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    Conflict(String),

    #[error("Resource has been deleted")]
    Gone,

    // Rate Limiting
    #[error("Too many requests")]
    RateLimitExceeded,

    #[error("Rate limit exceeded. Retry after {0} seconds")]
    RateLimitExceededWithRetry(u64),

    // Business Logic Errors
    #[error("Operation not allowed: {0}")]
    OperationNotAllowed(String),

    #[error("Content policy violation: {0}")]
    ContentPolicyViolation(String),

    #[error("Account suspended: {0}")]
    AccountSuspended(String),

    // Server Errors
    #[error("Internal server error")]
    InternalError,

    #[error("Database error")]
    DatabaseError,

    #[error("Cache error")]
    CacheError,

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    // OAuth Errors
    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Invalid OAuth state")]
    InvalidOAuthState,

    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),

    // File Upload Errors
    #[error("File too large: maximum size is {0} bytes")]
    FileTooLarge(usize),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File processing failed: {0}")]
    FileProcessingFailed(String),

    // WebSocket Errors
    #[error("WebSocket connection error: {0}")]
    WebSocketError(String),

    // Generic error with custom message
    #[error("{message}")]
    Custom {
        status: StatusCode,
        code: String,
        message: String,
    },
}

impl ApiError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 401 Unauthorized
            Self::Unauthorized
            | Self::InvalidCredentials
            | Self::TokenExpired
            | Self::InvalidToken
            | Self::RefreshTokenReuse
            | Self::InvalidSignature => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            Self::Forbidden | Self::InsufficientPermissions | Self::AccountSuspended(_) => {
                StatusCode::FORBIDDEN
            }

            // 400 Bad Request
            Self::ValidationError(_)
            | Self::InvalidInput(_)
            | Self::InvalidPublicKey
            | Self::OAuthError(_)
            | Self::InvalidOAuthState
            | Self::InvalidFileType(_) => StatusCode::BAD_REQUEST,

            // 404 Not Found
            Self::NotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::Conflict(_) => StatusCode::CONFLICT,

            // 410 Gone
            Self::Gone => StatusCode::GONE,

            // 413 Payload Too Large
            Self::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,

            // 422 Unprocessable Entity
            Self::OperationNotAllowed(_)
            | Self::ContentPolicyViolation(_)
            | Self::FileProcessingFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,

            // 429 Too Many Requests
            Self::RateLimitExceeded | Self::RateLimitExceededWithRetry(_) => {
                StatusCode::TOO_MANY_REQUESTS
            }

            // 500 Internal Server Error
            Self::InternalError
            | Self::DatabaseError
            | Self::CacheError
            | Self::CryptoError(_)
            | Self::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            // 502 Bad Gateway
            Self::OAuthProviderError(_) | Self::ExternalServiceError(_) => {
                StatusCode::BAD_GATEWAY
            }

            // 503 Service Unavailable
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,

            // WebSocket errors
            Self::WebSocketError(_) => StatusCode::BAD_REQUEST,

            // Custom error
            Self::Custom { status, .. } => *status,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::RefreshTokenReuse => "REFRESH_TOKEN_REUSE",
            Self::Forbidden => "FORBIDDEN",
            Self::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::InvalidSignature => "INVALID_SIGNATURE",
            Self::InvalidPublicKey => "INVALID_PUBLIC_KEY",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Gone => "GONE",
            Self::RateLimitExceeded | Self::RateLimitExceededWithRetry(_) => "RATE_LIMIT_EXCEEDED",
            Self::OperationNotAllowed(_) => "OPERATION_NOT_ALLOWED",
            Self::ContentPolicyViolation(_) => "CONTENT_POLICY_VIOLATION",
            Self::AccountSuspended(_) => "ACCOUNT_SUSPENDED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::DatabaseError => "DATABASE_ERROR",
            Self::CacheError => "CACHE_ERROR",
            Self::CryptoError(_) => "CRYPTO_ERROR",
            Self::StorageError(_) => "STORAGE_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            Self::OAuthError(_) => "OAUTH_ERROR",
            Self::InvalidOAuthState => "INVALID_OAUTH_STATE",
            Self::OAuthProviderError(_) => "OAUTH_PROVIDER_ERROR",
            Self::FileTooLarge(_) => "FILE_TOO_LARGE",
            Self::InvalidFileType(_) => "INVALID_FILE_TYPE",
            Self::FileProcessingFailed(_) => "FILE_PROCESSING_FAILED",
            Self::WebSocketError(_) => "WEBSOCKET_ERROR",
            Self::Custom { code, .. } => code.as_str(),
        }
    }

    /// Create a custom error with specific status code and message
    pub fn custom(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Custom {
            status,
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create a validation error from field errors
    pub fn validation_errors(errors: HashMap<String, Vec<String>>) -> Self {
        let message = errors
            .iter()
            .flat_map(|(field, errs)| errs.iter().map(move |e| format!("{}: {}", field, e)))
            .collect::<Vec<_>>()
            .join("; ");
        Self::ValidationError(message)
    }
}

/// Error response body
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Retry-after hint in seconds (for rate limiting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Generate a request ID for tracing
        let request_id = Uuid::new_v4().to_string();

        // Log server errors
        match &self {
            ApiError::InternalError
            | ApiError::DatabaseError
            | ApiError::CacheError
            | ApiError::CryptoError(_)
            | ApiError::StorageError(_) => {
                error!(
                    request_id = %request_id,
                    error = %self,
                    "Internal server error occurred"
                );
            }
            _ => {}
        }

        let retry_after = match &self {
            ApiError::RateLimitExceededWithRetry(seconds) => Some(*seconds),
            _ => None,
        };

        let status = self.status_code();
        let error_response = ErrorResponse {
            code: self.error_code().to_string(),
            message: self.to_string(),
            request_id: Some(request_id),
            details: None,
            retry_after,
        };

        let mut response = (status, Json(error_response)).into_response();

        // Add Retry-After header for rate limiting
        if let Some(seconds) = retry_after {
            response.headers_mut().insert(
                "Retry-After",
                seconds.to_string().parse().unwrap(),
            );
        }

        response
    }
}

// Conversion implementations for common error types

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = %err, "Database error");
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // Handle unique constraint violations
                if let Some(constraint) = db_err.constraint() {
                    return ApiError::Conflict(format!(
                        "Duplicate entry for constraint: {}",
                        constraint
                    ));
                }
                ApiError::DatabaseError
            }
            _ => ApiError::DatabaseError,
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        error!(error = %err, "Redis error");
        ApiError::CacheError
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => ApiError::TokenExpired,
            ErrorKind::InvalidToken
            | ErrorKind::InvalidSignature
            | ErrorKind::InvalidAlgorithm => ApiError::InvalidToken,
            _ => ApiError::InvalidToken,
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        let field_errors: HashMap<String, Vec<String>> = err
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("Invalid value for {}", field))
                    })
                    .collect();
                (field.to_string(), messages)
            })
            .collect();

        ApiError::validation_errors(field_errors)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        error!(error = %err, "IO error");
        ApiError::StorageError(err.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        error!(error = %err, "HTTP client error");
        ApiError::ExternalServiceError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        error!(error = %err, "Password hashing error");
        ApiError::CryptoError("Password processing failed".to_string())
    }
}

impl From<base64::DecodeError> for ApiError {
    fn from(_err: base64::DecodeError) -> Self {
        ApiError::InvalidInput("Invalid base64 encoding".to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::InvalidInput(format!("JSON parsing error: {}", err))
    }
}

/// Extension trait for Result to add context to errors
pub trait ResultExt<T> {
    fn with_not_found(self, resource: &str) -> ApiResult<T>;
    fn with_forbidden(self) -> ApiResult<T>;
}

impl<T, E: Into<ApiError>> ResultExt<T> for Result<T, E> {
    fn with_not_found(self, resource: &str) -> ApiResult<T> {
        self.map_err(|_| ApiError::NotFound(resource.to_string()))
    }

    fn with_forbidden(self) -> ApiResult<T> {
        self.map_err(|_| ApiError::Forbidden)
    }
}

/// Extension trait for Option to convert to ApiError
pub trait OptionExt<T> {
    fn ok_or_not_found(self, resource: &str) -> ApiResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, resource: &str) -> ApiResult<T> {
        self.ok_or_else(|| ApiError::NotFound(resource.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Forbidden.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::RateLimitExceeded.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(ApiError::InternalError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ApiError::Unauthorized.error_code(), "UNAUTHORIZED");
        assert_eq!(ApiError::TokenExpired.error_code(), "TOKEN_EXPIRED");
        assert_eq!(ApiError::NotFound("x".to_string()).error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_custom_error() {
        let err = ApiError::custom(
            StatusCode::IM_A_TEAPOT,
            "TEAPOT",
            "I'm a teapot"
        );
        assert_eq!(err.status_code(), StatusCode::IM_A_TEAPOT);
    }
}
