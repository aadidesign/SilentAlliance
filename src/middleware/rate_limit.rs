//! Rate limiting middleware
//!
//! Implements sliding window rate limiting using Redis.

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::infrastructure::cache::RateLimiter;
use crate::AppState;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per window
    pub limit: u32,
    /// Window duration
    pub window: Duration,
    /// Key prefix for this rate limit
    pub key_prefix: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            limit: 100,
            window: Duration::from_secs(60),
            key_prefix: "default".to_string(),
        }
    }
}

/// Extract rate limit key from request.
///
/// Prioritises authenticated identity from JWT claims. Falls back to
/// the peer IP address from `x-real-ip` (set by trusted reverse proxy),
/// then the *last* entry in `x-forwarded-for` (closest to the proxy),
/// and finally a generic fallback key.
///
/// IMPORTANT: `x-forwarded-for` is only trustworthy when your reverse
/// proxy strips/overwrites it. We use the *last* value (the one appended
/// by the proxy) rather than the first (which can be spoofed by the client).
fn extract_rate_limit_key(request: &Request<axum::body::Body>) -> String {
    // Try to get identity from token (set by auth middleware)
    if let Some(claims) = request.extensions().get::<crate::domain::services::auth::Claims>() {
        return format!("identity:{}", claims.sub);
    }

    // Prefer x-real-ip (set by trusted proxy like nginx)
    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|h| h.to_str().ok())
    {
        let ip = real_ip.trim();
        if !ip.is_empty() {
            return format!("ip:{}", ip);
        }
    }

    // Fall back to x-forwarded-for — use the LAST entry (proxy-appended, harder to spoof)
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(ip) = forwarded.rsplit(',').next() {
            let ip = ip.trim();
            if !ip.is_empty() {
                return format!("ip:{}", ip);
            }
        }
    }

    "ip:unknown".to_string()
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if !state.settings.rate_limit.enabled {
        return Ok(next.run(request).await);
    }

    let key = extract_rate_limit_key(&request);
    let path = request.uri().path();

    // Determine rate limit based on endpoint
    let (limit, window) = get_rate_limit_for_path(path, &state);

    let rate_limiter = RateLimiter::new(&state.redis);
    let full_key = format!("{}:{}", path_to_key(path), key);

    match rate_limiter.check_and_increment(&full_key, limit, window).await {
        Ok((is_allowed, remaining, reset_time)) => {
            let mut response = if is_allowed {
                next.run(request).await
            } else {
                warn!(key = %key, path = %path, "Rate limit exceeded");
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    format!("Rate limit exceeded. Retry after {} seconds", reset_time),
                ));
            };

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Reset", reset_time.to_string().parse().unwrap());

            Ok(response)
        }
        Err(e) => {
            // On Redis error, allow the request but emit a strong warning.
            // In a high-security deployment, you may want to reject instead.
            error!(error = %e, path = %path, key = %key, "Rate limiter Redis error — request allowed without rate check");
            Ok(next.run(request).await)
        }
    }
}

/// Get rate limit for a specific path
fn get_rate_limit_for_path(path: &str, state: &Arc<AppState>) -> (u32, Duration) {
    let settings = &state.settings.rate_limit;

    // Authentication endpoints have stricter limits
    if path.starts_with("/api/v1/auth") {
        return (settings.auth_rps, Duration::from_secs(1));
    }

    // Media upload has lower limits
    if path.contains("/media/upload") {
        return (settings.expensive_rpm, Duration::from_secs(60));
    }

    // General rate limit
    (settings.general_rps, Duration::from_secs(1))
}

/// Convert path to a rate limit key
fn path_to_key(path: &str) -> String {
    // Normalize path for rate limiting (remove IDs)
    let parts: Vec<&str> = path.split('/').collect();
    let normalized: Vec<&str> = parts
        .iter()
        .map(|&p| {
            // Replace UUIDs with placeholder
            if p.len() == 36 && p.chars().filter(|c| *c == '-').count() == 4 {
                ":id"
            } else {
                p
            }
        })
        .collect();

    normalized.join("/")
}

/// Per-endpoint rate limit configuration
pub struct EndpointRateLimits;

impl EndpointRateLimits {
    /// Get rate limits for specific endpoints
    pub fn get(endpoint: &str) -> RateLimitConfig {
        match endpoint {
            // Very strict for auth
            "/api/v1/auth/login" => RateLimitConfig {
                limit: 5,
                window: Duration::from_secs(60),
                key_prefix: "auth:login".to_string(),
            },
            "/api/v1/auth/register" => RateLimitConfig {
                limit: 3,
                window: Duration::from_secs(3600), // 3 per hour
                key_prefix: "auth:register".to_string(),
            },
            // Posting limits
            "/api/v1/posts" => RateLimitConfig {
                limit: 10,
                window: Duration::from_secs(3600),
                key_prefix: "posts:create".to_string(),
            },
            "/api/v1/comments" => RateLimitConfig {
                limit: 30,
                window: Duration::from_secs(3600),
                key_prefix: "comments:create".to_string(),
            },
            // Default
            _ => RateLimitConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_normalization() {
        assert_eq!(
            path_to_key("/api/v1/posts/123e4567-e89b-12d3-a456-426614174000"),
            "/api/v1/posts/:id"
        );
        assert_eq!(path_to_key("/api/v1/spaces"), "/api/v1/spaces");
    }
}
