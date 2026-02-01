//! API layer for SilentAlliance
//!
//! This module contains all HTTP handlers, route definitions,
//! request/response types, and API-specific logic.

mod routes;
pub mod auth;
pub mod identity;
pub mod spaces;
pub mod posts;
pub mod comments;
pub mod votes;
pub mod messages;
pub mod media;
pub mod notifications;
pub mod moderation;
pub mod feed;
pub mod health;

pub use routes::create_router;

/// Common response types
pub mod response {
    use serde::{Deserialize, Serialize};

    /// Success response wrapper
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SuccessResponse<T> {
        pub success: bool,
        pub data: T,
    }

    impl<T> SuccessResponse<T> {
        pub fn new(data: T) -> Self {
            Self {
                success: true,
                data,
            }
        }
    }

    /// Message response for simple operations
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MessageResponse {
        pub success: bool,
        pub message: String,
    }

    impl MessageResponse {
        pub fn new(message: impl Into<String>) -> Self {
            Self {
                success: true,
                message: message.into(),
            }
        }
    }

    /// Deletion response
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DeleteResponse {
        pub success: bool,
        pub deleted: bool,
    }

    impl DeleteResponse {
        pub fn new(deleted: bool) -> Self {
            Self {
                success: true,
                deleted,
            }
        }
    }
}

/// Request extractors and utilities
pub mod extractors {
    use axum::{
        async_trait,
        extract::{FromRequestParts, Query},
        http::request::Parts,
    };
    use serde::de::DeserializeOwned;
    use validator::Validate;

    use crate::domain::entities::PaginationParams;
    use crate::errors::ApiError;

    /// Validated query parameters extractor
    pub struct ValidatedQuery<T>(pub T);

    #[async_trait]
    impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
    where
        S: Send + Sync,
        T: DeserializeOwned + Validate,
    {
        type Rejection = ApiError;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let Query(value) = Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|e| ApiError::InvalidInput(e.to_string()))?;

            value.validate()?;
            Ok(ValidatedQuery(value))
        }
    }

    /// Pagination query extractor
    pub struct Pagination(pub PaginationParams);

    #[async_trait]
    impl<S> FromRequestParts<S> for Pagination
    where
        S: Send + Sync,
    {
        type Rejection = ApiError;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let Query(params) = Query::<PaginationParams>::from_request_parts(parts, state)
                .await
                .map_err(|e| ApiError::InvalidInput(e.to_string()))?;

            params.validate()?;
            Ok(Pagination(params))
        }
    }
}
