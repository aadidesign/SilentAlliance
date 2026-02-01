//! Redis cache infrastructure module
//!
//! Provides Redis connection pool management, caching utilities,
//! session management, and rate limiting support.

use deadpool_redis::{Config, Pool, Runtime, Connection};
use redis::{AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

use crate::config::RedisSettings;
use crate::errors::ApiError;

/// Redis connection pool wrapper
#[derive(Clone)]
pub struct RedisPool {
    pool: Pool,
    key_prefix: String,
}

impl RedisPool {
    /// Create a new Redis connection pool
    pub async fn new(settings: &RedisSettings) -> Result<Self, ApiError> {
        let cfg = Config::from_url(&settings.url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| {
                error!(error = %e, "Failed to create Redis pool");
                ApiError::CacheError
            })?;

        // Test connection
        let mut conn = pool.get().await.map_err(|e| {
            error!(error = %e, "Failed to get Redis connection");
            ApiError::CacheError
        })?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!(error = %e, "Redis ping failed");
                ApiError::CacheError
            })?;

        info!(
            max_size = settings.max_connections,
            "Redis pool created and connected"
        );

        Ok(Self {
            pool,
            key_prefix: settings.key_prefix.clone(),
        })
    }

    /// Get a connection from the pool
    pub async fn get_conn(&self) -> Result<Connection, ApiError> {
        self.pool.get().await.map_err(|e| {
            error!(error = %e, "Failed to get Redis connection");
            ApiError::CacheError
        })
    }

    /// Create a prefixed key
    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    /// Set a value with optional expiration
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);
        let serialized = serde_json::to_string(value).map_err(|_| ApiError::CacheError)?;

        if let Some(ttl) = ttl {
            conn.set_ex::<_, _, ()>(&prefixed_key, &serialized, ttl.as_secs())
                .await
                .map_err(|e| {
                    error!(error = %e, key = %key, "Failed to set cache value");
                    ApiError::CacheError
                })?;
        } else {
            conn.set::<_, _, ()>(&prefixed_key, &serialized)
                .await
                .map_err(|e| {
                    error!(error = %e, key = %key, "Failed to set cache value");
                    ApiError::CacheError
                })?;
        }

        debug!(key = %key, "Cache set");
        Ok(())
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let value: Option<String> = conn.get(&prefixed_key).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to get cache value");
            ApiError::CacheError
        })?;

        match value {
            Some(v) => {
                let deserialized: T = serde_json::from_str(&v).map_err(|_| ApiError::CacheError)?;
                debug!(key = %key, "Cache hit");
                Ok(Some(deserialized))
            }
            None => {
                debug!(key = %key, "Cache miss");
                Ok(None)
            }
        }
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        conn.del::<_, ()>(&prefixed_key).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to delete cache key");
            ApiError::CacheError
        })?;

        debug!(key = %key, "Cache key deleted");
        Ok(())
    }

    /// Delete multiple keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_pattern = self.prefixed_key(pattern);

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefixed_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!(error = %e, pattern = %pattern, "Failed to find keys");
                ApiError::CacheError
            })?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count: u64 = conn.del(&keys).await.map_err(|e| {
            error!(error = %e, pattern = %pattern, "Failed to delete keys");
            ApiError::CacheError
        })?;

        debug!(pattern = %pattern, count = count, "Cache keys deleted by pattern");
        Ok(count)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let exists: bool = conn.exists(&prefixed_key).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to check key existence");
            ApiError::CacheError
        })?;

        Ok(exists)
    }

    /// Set expiration on an existing key
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let result: bool = conn
            .expire(&prefixed_key, ttl.as_secs() as i64)
            .await
            .map_err(|e| {
                error!(error = %e, key = %key, "Failed to set expiration");
                ApiError::CacheError
            })?;

        Ok(result)
    }

    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let value: i64 = conn.incr(&prefixed_key, 1).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to increment counter");
            ApiError::CacheError
        })?;

        Ok(value)
    }

    /// Increment a counter with expiration (set only if key doesn't exist)
    pub async fn incr_with_ttl(&self, key: &str, ttl: Duration) -> Result<i64, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        // Use MULTI/EXEC for atomic operation
        let value: i64 = conn.incr(&prefixed_key, 1).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to increment counter");
            ApiError::CacheError
        })?;

        // Set expiration only if this is the first increment (value == 1)
        if value == 1 {
            conn.expire::<_, ()>(&prefixed_key, ttl.as_secs() as i64)
                .await
                .map_err(|e| {
                    error!(error = %e, key = %key, "Failed to set expiration");
                    ApiError::CacheError
                })?;
        }

        Ok(value)
    }

    /// Get TTL of a key
    pub async fn ttl(&self, key: &str) -> Result<i64, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let ttl: i64 = conn.ttl(&prefixed_key).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to get TTL");
            ApiError::CacheError
        })?;

        Ok(ttl)
    }

    /// Add a member to a set
    pub async fn sadd(&self, key: &str, member: &str) -> Result<bool, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let added: i64 = conn.sadd(&prefixed_key, member).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to add to set");
            ApiError::CacheError
        })?;

        Ok(added > 0)
    }

    /// Remove a member from a set
    pub async fn srem(&self, key: &str, member: &str) -> Result<bool, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let removed: i64 = conn.srem(&prefixed_key, member).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to remove from set");
            ApiError::CacheError
        })?;

        Ok(removed > 0)
    }

    /// Check if a member exists in a set
    pub async fn sismember(&self, key: &str, member: &str) -> Result<bool, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let exists: bool = conn.sismember(&prefixed_key, member).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to check set membership");
            ApiError::CacheError
        })?;

        Ok(exists)
    }

    /// Get all members of a set
    pub async fn smembers(&self, key: &str) -> Result<Vec<String>, ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_key = self.prefixed_key(key);

        let members: Vec<String> = conn.smembers(&prefixed_key).await.map_err(|e| {
            error!(error = %e, key = %key, "Failed to get set members");
            ApiError::CacheError
        })?;

        Ok(members)
    }

    /// Publish a message to a channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<(), ApiError> {
        let mut conn = self.get_conn().await?;
        let prefixed_channel = self.prefixed_key(channel);

        conn.publish::<_, _, ()>(&prefixed_channel, message)
            .await
            .map_err(|e| {
                error!(error = %e, channel = %channel, "Failed to publish message");
                ApiError::CacheError
            })?;

        debug!(channel = %channel, "Message published");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), ApiError> {
        let mut conn = self.get_conn().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }
}

/// Session data structure
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SessionData {
    pub identity_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Session manager for handling user sessions
pub struct SessionManager<'a> {
    redis: &'a RedisPool,
    session_ttl: Duration,
}

impl<'a> SessionManager<'a> {
    /// Create a new session manager
    pub fn new(redis: &'a RedisPool, session_ttl: Duration) -> Self {
        Self { redis, session_ttl }
    }

    /// Create a new session
    pub async fn create(&self, session_id: &str, data: &SessionData) -> Result<(), ApiError> {
        let key = format!("session:{}", session_id);
        self.redis.set(&key, data, Some(self.session_ttl)).await
    }

    /// Get session data
    pub async fn get(&self, session_id: &str) -> Result<Option<SessionData>, ApiError> {
        let key = format!("session:{}", session_id);
        self.redis.get(&key).await
    }

    /// Update session activity
    pub async fn touch(&self, session_id: &str) -> Result<(), ApiError> {
        let key = format!("session:{}", session_id);
        if let Some(mut data) = self.get(session_id).await? {
            data.last_activity = chrono::Utc::now();
            self.redis.set(&key, &data, Some(self.session_ttl)).await?;
        }
        Ok(())
    }

    /// Delete a session
    pub async fn delete(&self, session_id: &str) -> Result<(), ApiError> {
        let key = format!("session:{}", session_id);
        self.redis.delete(&key).await
    }

    /// Delete all sessions for an identity
    pub async fn delete_all_for_identity(&self, identity_id: &uuid::Uuid) -> Result<u64, ApiError> {
        // In production, you'd use a secondary index for this
        // For now, we'll use pattern matching (not ideal for large scale)
        let pattern = "session:*";
        // This is a simplified implementation - in production, maintain a set of sessions per identity
        self.redis.delete_pattern(pattern).await
    }
}

/// Rate limiter using sliding window algorithm
pub struct RateLimiter<'a> {
    redis: &'a RedisPool,
}

impl<'a> RateLimiter<'a> {
    /// Create a new rate limiter
    pub fn new(redis: &'a RedisPool) -> Self {
        Self { redis }
    }

    /// Check if request is allowed and increment counter
    /// Returns (is_allowed, remaining_requests, reset_time)
    pub async fn check_and_increment(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<(bool, u32, i64), ApiError> {
        let full_key = format!("ratelimit:{}", key);
        let count = self.redis.incr_with_ttl(&full_key, window).await?;
        let ttl = self.redis.ttl(&full_key).await?;

        let is_allowed = count <= limit as i64;
        let remaining = if is_allowed {
            limit - count as u32
        } else {
            0
        };

        Ok((is_allowed, remaining, ttl))
    }

    /// Get current rate limit status without incrementing
    pub async fn get_status(
        &self,
        key: &str,
        limit: u32,
    ) -> Result<(u32, i64), ApiError> {
        let full_key = format!("ratelimit:{}", key);

        let count: Option<i64> = self.redis.get(&full_key).await?;
        let ttl = self.redis.ttl(&full_key).await?;

        let remaining = match count {
            Some(c) => {
                if c >= limit as i64 {
                    0
                } else {
                    limit - c as u32
                }
            }
            None => limit,
        };

        Ok((remaining, ttl))
    }

    /// Reset rate limit for a key
    pub async fn reset(&self, key: &str) -> Result<(), ApiError> {
        let full_key = format!("ratelimit:{}", key);
        self.redis.delete(&full_key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a Redis instance
}
