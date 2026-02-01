//! # SilentAlliance
//!
//! An anonymous, privacy-first social discussion platform backend API built in Rust.
//!
//! ## Features
//!
//! - **Pseudonymous Identity**: Ed25519 keypair-based identity system
//! - **OAuth 2.0 with PKCE**: Secure authentication flow
//! - **JWT Authentication**: RS256 signed tokens with refresh token rotation
//! - **End-to-End Encryption**: X25519 key exchange with ChaCha20-Poly1305
//! - **Reddit-style Features**: Spaces, posts, threaded comments, voting
//! - **Real-time Notifications**: WebSocket-based live updates
//! - **Content Moderation**: Reporting and moderation system
//!
//! ## Architecture
//!
//! The application follows a clean architecture pattern:
//!
//! - `api`: HTTP handlers and route definitions
//! - `domain`: Business entities, services, and repository traits
//! - `infrastructure`: Database, cache, crypto, and storage implementations
//! - `middleware`: Authentication, rate limiting, and security middleware

use std::sync::Arc;

pub mod api;
pub mod config;
pub mod domain;
pub mod errors;
pub mod infrastructure;
pub mod middleware;
pub mod websocket;
pub mod jobs;

use config::Settings;
use infrastructure::{
    database::DatabasePool,
    cache::RedisPool,
    crypto::CryptoService,
    storage::StorageService,
};

/// Shared application state accessible from all handlers
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: DatabasePool,
    /// Redis connection pool for caching and sessions
    pub redis: RedisPool,
    /// Cryptographic service for JWT, hashing, and encryption
    pub crypto: CryptoService,
    /// File storage service
    pub storage: StorageService,
    /// Application settings
    pub settings: Settings,
}

impl AppState {
    /// Get a reference to the database pool
    pub fn db(&self) -> &DatabasePool {
        &self.db
    }

    /// Get a reference to the Redis pool
    pub fn redis(&self) -> &RedisPool {
        &self.redis
    }

    /// Get a reference to the crypto service
    pub fn crypto(&self) -> &CryptoService {
        &self.crypto
    }

    /// Get a reference to the storage service
    pub fn storage(&self) -> &StorageService {
        &self.storage
    }
}

/// Type alias for the shared state wrapped in Arc
pub type SharedState = Arc<AppState>;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::errors::{ApiError, ApiResult};
    pub use crate::AppState;
    pub use crate::SharedState;
    pub use axum::{
        extract::{Json, Path, Query, State},
        http::StatusCode,
        response::IntoResponse,
    };
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
    pub use chrono::{DateTime, Utc};
    pub use tracing::{debug, error, info, instrument, warn};
    pub use validator::Validate;
}
