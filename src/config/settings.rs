//! Application settings and configuration management
//!
//! This module provides strongly-typed configuration loading from environment variables
//! with validation and sensible defaults.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Main application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerSettings,
    /// Database configuration
    pub database: DatabaseSettings,
    /// Redis configuration
    pub redis: RedisSettings,
    /// Cryptographic settings
    pub crypto: CryptoSettings,
    /// JWT settings
    pub jwt: JwtSettings,
    /// OAuth provider settings
    pub oauth: OAuthSettings,
    /// Storage settings
    pub storage: StorageSettings,
    /// Rate limiting settings
    pub rate_limit: RateLimitSettings,
    /// CORS settings
    pub cors: CorsSettings,
}

impl Settings {
    /// Load settings from environment variables
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            server: ServerSettings::from_env()?,
            database: DatabaseSettings::from_env()?,
            redis: RedisSettings::from_env()?,
            crypto: CryptoSettings::from_env()?,
            jwt: JwtSettings::from_env()?,
            oauth: OAuthSettings::from_env()?,
            storage: StorageSettings::from_env()?,
            rate_limit: RateLimitSettings::from_env()?,
            cors: CorsSettings::from_env()?,
        })
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.server.environment == "production"
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Environment (development, staging, production)
    pub environment: String,
    /// Request body size limit in bytes
    pub body_limit: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl ServerSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?,
            environment: env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
            body_limit: env::var("BODY_LIMIT")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()
                .map_err(|_| ConfigError::InvalidValue("BODY_LIMIT".to_string()))?,
            request_timeout: env::var("REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("REQUEST_TIMEOUT".to_string()))?,
        })
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    /// PostgreSQL connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,
}

impl DatabaseSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingRequired("DATABASE_URL".to_string()))?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MAX_CONNECTIONS".to_string()))?,
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MIN_CONNECTIONS".to_string()))?,
            connect_timeout: env::var("DATABASE_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_CONNECT_TIMEOUT".to_string()))?,
            idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_IDLE_TIMEOUT".to_string()))?,
            max_lifetime: env::var("DATABASE_MAX_LIFETIME")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MAX_LIFETIME".to_string()))?,
        })
    }

    /// Get connection timeout as Duration
    pub fn connect_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.connect_timeout)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.idle_timeout)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime_duration(&self) -> Duration {
        Duration::from_secs(self.max_lifetime)
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSettings {
    /// Redis connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Key prefix for namespacing
    pub key_prefix: String,
}

impl RedisSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            url: env::var("REDIS_URL")
                .map_err(|_| ConfigError::MissingRequired("REDIS_URL".to_string()))?,
            max_connections: env::var("REDIS_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("REDIS_MAX_CONNECTIONS".to_string()))?,
            connect_timeout: env::var("REDIS_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("REDIS_CONNECT_TIMEOUT".to_string()))?,
            key_prefix: env::var("REDIS_KEY_PREFIX")
                .unwrap_or_else(|_| "silentalliance:".to_string()),
        })
    }
}

/// Cryptographic settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSettings {
    /// Master encryption key (base64 encoded, 32 bytes)
    pub master_key: String,
    /// Argon2 memory cost in KB
    pub argon2_memory_cost: u32,
    /// Argon2 time cost (iterations)
    pub argon2_time_cost: u32,
    /// Argon2 parallelism
    pub argon2_parallelism: u32,
}

impl CryptoSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            master_key: env::var("MASTER_KEY")
                .map_err(|_| ConfigError::MissingRequired("MASTER_KEY".to_string()))?,
            argon2_memory_cost: env::var("ARGON2_MEMORY_COST")
                .unwrap_or_else(|_| "65536".to_string()) // 64MB
                .parse()
                .map_err(|_| ConfigError::InvalidValue("ARGON2_MEMORY_COST".to_string()))?,
            argon2_time_cost: env::var("ARGON2_TIME_COST")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("ARGON2_TIME_COST".to_string()))?,
            argon2_parallelism: env::var("ARGON2_PARALLELISM")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("ARGON2_PARALLELISM".to_string()))?,
        })
    }
}

/// JWT settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSettings {
    /// RSA private key in PEM format (for signing)
    pub private_key: String,
    /// RSA public key in PEM format (for verification)
    pub public_key: String,
    /// Access token expiration in seconds
    pub access_token_expiry: i64,
    /// Refresh token expiration in seconds
    pub refresh_token_expiry: i64,
    /// Token issuer
    pub issuer: String,
    /// Token audience
    pub audience: String,
}

impl JwtSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            private_key: env::var("JWT_PRIVATE_KEY")
                .map_err(|_| ConfigError::MissingRequired("JWT_PRIVATE_KEY".to_string()))?,
            public_key: env::var("JWT_PUBLIC_KEY")
                .map_err(|_| ConfigError::MissingRequired("JWT_PUBLIC_KEY".to_string()))?,
            access_token_expiry: env::var("JWT_ACCESS_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()
                .map_err(|_| ConfigError::InvalidValue("JWT_ACCESS_TOKEN_EXPIRY".to_string()))?,
            refresh_token_expiry: env::var("JWT_REFRESH_TOKEN_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .map_err(|_| ConfigError::InvalidValue("JWT_REFRESH_TOKEN_EXPIRY".to_string()))?,
            issuer: env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "silentalliance".to_string()),
            audience: env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "silentalliance-api".to_string()),
        })
    }

    /// Get access token expiry as Duration
    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.access_token_expiry as u64)
    }

    /// Get refresh token expiry as Duration
    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.refresh_token_expiry as u64)
    }
}

/// OAuth provider settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSettings {
    /// GitHub OAuth settings
    pub github: Option<OAuthProvider>,
    /// Discord OAuth settings
    pub discord: Option<OAuthProvider>,
    /// OAuth state HMAC secret
    pub state_secret: String,
    /// Authorization code expiry in seconds
    pub code_expiry: i64,
}

impl OAuthSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            github: OAuthProvider::github_from_env().ok(),
            discord: OAuthProvider::discord_from_env().ok(),
            state_secret: env::var("OAUTH_STATE_SECRET")
                .map_err(|_| ConfigError::MissingRequired("OAUTH_STATE_SECRET".to_string()))?,
            code_expiry: env::var("OAUTH_CODE_EXPIRY")
                .unwrap_or_else(|_| "300".to_string()) // 5 minutes
                .parse()
                .map_err(|_| ConfigError::InvalidValue("OAUTH_CODE_EXPIRY".to_string()))?,
        })
    }
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Authorization URL
    pub auth_url: String,
    /// Token URL
    pub token_url: String,
    /// User info URL
    pub userinfo_url: String,
    /// Scopes to request
    pub scopes: Vec<String>,
}

impl OAuthProvider {
    fn github_from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            client_id: env::var("GITHUB_CLIENT_ID")
                .map_err(|_| ConfigError::MissingRequired("GITHUB_CLIENT_ID".to_string()))?,
            client_secret: env::var("GITHUB_CLIENT_SECRET")
                .map_err(|_| ConfigError::MissingRequired("GITHUB_CLIENT_SECRET".to_string()))?,
            redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/oauth/callback/github".to_string()),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            userinfo_url: "https://api.github.com/user".to_string(),
            scopes: vec!["read:user".to_string()],
        })
    }

    fn discord_from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            client_id: env::var("DISCORD_CLIENT_ID")
                .map_err(|_| ConfigError::MissingRequired("DISCORD_CLIENT_ID".to_string()))?,
            client_secret: env::var("DISCORD_CLIENT_SECRET")
                .map_err(|_| ConfigError::MissingRequired("DISCORD_CLIENT_SECRET".to_string()))?,
            redirect_uri: env::var("DISCORD_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8080/api/v1/auth/oauth/callback/discord".to_string()),
            auth_url: "https://discord.com/api/oauth2/authorize".to_string(),
            token_url: "https://discord.com/api/oauth2/token".to_string(),
            userinfo_url: "https://discord.com/api/users/@me".to_string(),
            scopes: vec!["identify".to_string()],
        })
    }
}

/// Storage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    /// Storage type (local, s3)
    pub storage_type: String,
    /// Local storage path
    pub local_path: String,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Allowed MIME types
    pub allowed_mime_types: Vec<String>,
    /// S3 bucket name (if using S3)
    pub s3_bucket: Option<String>,
    /// S3 region (if using S3)
    pub s3_region: Option<String>,
    /// S3 endpoint (for S3-compatible storage)
    pub s3_endpoint: Option<String>,
}

impl StorageSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            storage_type: env::var("STORAGE_TYPE")
                .unwrap_or_else(|_| "local".to_string()),
            local_path: env::var("STORAGE_LOCAL_PATH")
                .unwrap_or_else(|_| "./uploads".to_string()),
            max_file_size: env::var("STORAGE_MAX_FILE_SIZE")
                .unwrap_or_else(|_| "52428800".to_string()) // 50MB
                .parse()
                .map_err(|_| ConfigError::InvalidValue("STORAGE_MAX_FILE_SIZE".to_string()))?,
            allowed_mime_types: env::var("STORAGE_ALLOWED_MIME_TYPES")
                .unwrap_or_else(|_| "image/jpeg,image/png,image/gif,image/webp".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            s3_bucket: env::var("S3_BUCKET").ok(),
            s3_region: env::var("S3_REGION").ok(),
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
        })
    }
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second for general endpoints
    pub general_rps: u32,
    /// Burst size for general endpoints
    pub general_burst: u32,
    /// Requests per second for authentication endpoints
    pub auth_rps: u32,
    /// Burst size for authentication endpoints
    pub auth_burst: u32,
    /// Requests per minute for expensive operations
    pub expensive_rpm: u32,
}

impl RateLimitSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            enabled: env::var("RATE_LIMIT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            general_rps: env::var("RATE_LIMIT_GENERAL_RPS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_GENERAL_RPS".to_string()))?,
            general_burst: env::var("RATE_LIMIT_GENERAL_BURST")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_GENERAL_BURST".to_string()))?,
            auth_rps: env::var("RATE_LIMIT_AUTH_RPS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_AUTH_RPS".to_string()))?,
            auth_burst: env::var("RATE_LIMIT_AUTH_BURST")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_AUTH_BURST".to_string()))?,
            expensive_rpm: env::var("RATE_LIMIT_EXPENSIVE_RPM")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_EXPENSIVE_RPM".to_string()))?,
        })
    }
}

/// CORS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsSettings {
    /// Allowed origins (comma-separated or * for all)
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age in seconds
    pub max_age: u64,
}

impl CorsSettings {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_methods: env::var("CORS_ALLOWED_METHODS")
                .unwrap_or_else(|_| "GET,POST,PUT,PATCH,DELETE,OPTIONS".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_headers: env::var("CORS_ALLOWED_HEADERS")
                .unwrap_or_else(|_| "Content-Type,Authorization,X-Request-Id".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            exposed_headers: env::var("CORS_EXPOSED_HEADERS")
                .unwrap_or_else(|_| "X-Request-Id,X-RateLimit-Remaining".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allow_credentials: env::var("CORS_ALLOW_CREDENTIALS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_age: env::var("CORS_MAX_AGE")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("CORS_MAX_AGE".to_string()))?,
        })
    }
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value for: {0}")]
    InvalidValue(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_settings_defaults() {
        // Clear relevant env vars for isolated testing
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("RUST_ENV");

        let settings = ServerSettings::from_env().unwrap();
        assert_eq!(settings.host, "0.0.0.0");
        assert_eq!(settings.port, 8080);
        assert_eq!(settings.environment, "development");
    }
}
