//! Security middleware
//!
//! Adds security headers to all responses.

use axum::{
    http::{Request, header},
    middleware::Next,
    response::Response,
};

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        "DENY".parse().unwrap(),
    );

    // Enable XSS filter in browsers
    headers.insert(
        "X-XSS-Protection",
        "1; mode=block".parse().unwrap(),
    );

    // Content Security Policy
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
            .parse()
            .unwrap(),
    );

    // Referrer Policy
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Permissions Policy (formerly Feature-Policy)
    headers.insert(
        "Permissions-Policy",
        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
            .parse()
            .unwrap(),
    );

    // Cache control for API responses
    if !headers.contains_key(header::CACHE_CONTROL) {
        headers.insert(
            header::CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, proxy-revalidate".parse().unwrap(),
        );
    }

    // Prevent caching of sensitive data
    headers.insert(
        header::PRAGMA,
        "no-cache".parse().unwrap(),
    );

    headers.insert(
        header::EXPIRES,
        "0".parse().unwrap(),
    );

    response
}

/// HSTS middleware (for production with HTTPS)
pub async fn hsts_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    // HTTP Strict Transport Security
    // max-age=31536000 (1 year), includeSubDomains, preload
    response.headers_mut().insert(
        header::STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
    );

    response
}

/// Request ID middleware for tracing
pub async fn request_id_middleware(
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Generate or extract request ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Store in extensions for logging
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Add to response headers
    response.headers_mut().insert(
        "X-Request-Id",
        request_id.parse().unwrap(),
    );

    response
}

/// Request ID wrapper
#[derive(Debug, Clone)]
pub struct RequestId(pub String);
