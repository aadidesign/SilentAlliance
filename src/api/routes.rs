//! API route definitions
//!
//! Combines all API modules into a single router with middleware.

use axum::{
    middleware,
    routing::{get, post, put, patch, delete},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
    limit::RequestBodyLimitLayer,
};
use std::time::Duration;

use crate::middleware::{
    auth::auth_middleware,
    rate_limit::rate_limit_middleware,
    security::security_headers_middleware,
};
use crate::AppState;

use super::{auth, identity, spaces, posts, comments, votes, messages, media, notifications, moderation, feed, health};

/// Create the main application router with all routes and middleware
pub fn create_router(state: Arc<AppState>) -> Router {
    // Auth middleware layer for protected routes
    let require_auth = middleware::from_fn_with_state(state.clone(), auth_middleware);

    // Rate limiting layer
    let rate_limit = middleware::from_fn_with_state(state.clone(), rate_limit_middleware);

    // Build API v1 routes
    let api_v1 = Router::new()
        // Health check (no auth required)
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Authentication routes (rate limited, no auth required)
        .nest("/auth", auth_routes())
        // Identity routes — public reads, auth-protected writes
        .nest("/identity", identity_routes())
        // Space routes — public reads, auth-protected writes
        .nest("/spaces", spaces_routes())
        // Post routes — public reads, auth-protected writes
        .nest("/posts", posts_routes())
        // Comment routes — public reads, auth-protected writes
        .nest("/comments", comments_routes())
        // Message routes (all require auth — E2E encrypted)
        .nest("/messages", messages_routes().layer(require_auth.clone()))
        // Media routes — upload requires auth, reads are public
        .nest("/media", media_routes())
        // Notification routes (all require auth)
        .nest("/notifications", notifications_routes().layer(require_auth.clone()))
        // Feed routes — personalized requires auth, public feeds don't
        .nest("/feed", feed_routes())
        // Moderation routes (all require auth + moderator check inside handlers)
        .nest("/moderation", moderation_routes().layer(require_auth.clone()))
        // Apply rate limiting to all API routes
        .layer(rate_limit);

    // Build the main router
    Router::new()
        .nest("/api/v1", api_v1)
        .layer(
            ServiceBuilder::new()
                // Add security headers
                .layer(middleware::from_fn(security_headers_middleware))
                // Request tracing
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                            )
                        })
                )
                // Response compression
                .layer(CompressionLayer::new())
                // Request body size limit
                .layer(RequestBodyLimitLayer::new(state.settings.server.body_limit))
                // CORS
                .layer(build_cors_layer(&state.settings.cors))
        )
        .with_state(state)
}

/// Build CORS layer from settings
fn build_cors_layer(settings: &crate::config::CorsSettings) -> CorsLayer {
    use axum::http::{HeaderName, HeaderValue, Method};

    let mut cors = CorsLayer::new();

    // Set allowed origins — warn on any that fail to parse so admins notice misconfiguration
    if settings.allowed_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<HeaderValue> = settings
            .allowed_origins
            .iter()
            .filter_map(|o| {
                o.parse::<HeaderValue>().map_err(|e| {
                    tracing::warn!(origin = %o, error = %e, "Invalid CORS origin ignored");
                }).ok()
            })
            .collect();
        cors = cors.allow_origin(origins);
    }

    // Set allowed methods — warn on invalid entries
    let methods: Vec<Method> = settings
        .allowed_methods
        .iter()
        .filter_map(|m| {
            m.parse::<Method>().map_err(|e| {
                tracing::warn!(method = %m, error = %e, "Invalid CORS method ignored");
            }).ok()
        })
        .collect();
    cors = cors.allow_methods(methods);

    // Set allowed headers — warn on invalid entries
    let headers: Vec<HeaderName> = settings
        .allowed_headers
        .iter()
        .filter_map(|h| {
            h.parse::<HeaderName>().map_err(|e| {
                tracing::warn!(header = %h, error = %e, "Invalid CORS header ignored");
            }).ok()
        })
        .collect();
    cors = cors.allow_headers(headers);

    // Set exposed headers — warn on invalid entries
    let exposed: Vec<HeaderName> = settings
        .exposed_headers
        .iter()
        .filter_map(|h| {
            h.parse::<HeaderName>().map_err(|e| {
                tracing::warn!(header = %h, error = %e, "Invalid CORS exposed header ignored");
            }).ok()
        })
        .collect();
    cors = cors.expose_headers(exposed);

    // Allow credentials
    if settings.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    // Max age
    cors = cors.max_age(Duration::from_secs(settings.max_age));

    cors
}

/// Authentication routes
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Registration and login
        .route("/register", post(auth::handlers::register))
        .route("/challenge", post(auth::handlers::get_challenge))
        .route("/login", post(auth::handlers::login))
        // Token management
        .route("/refresh", post(auth::handlers::refresh_token))
        .route("/logout", post(auth::handlers::logout))
        .route("/logout-all", post(auth::handlers::logout_all))
        // OAuth 2.0 PKCE
        .route("/oauth/authorize", get(auth::handlers::oauth_authorize))
        .route("/oauth/callback/:provider", get(auth::handlers::oauth_callback))
        .route("/oauth/token", post(auth::handlers::oauth_token_exchange))
}

/// Identity routes
fn identity_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Current user
        .route("/me", get(identity::handlers::get_current))
        .route("/me", patch(identity::handlers::update_current))
        .route("/me/sessions", get(identity::handlers::list_sessions))
        .route("/me/sessions/:id", delete(identity::handlers::revoke_session))
        // Public identity lookup
        .route("/:id", get(identity::handlers::get_by_id))
        .route("/:id/posts", get(identity::handlers::get_posts))
        .route("/:id/comments", get(identity::handlers::get_comments))
        .route("/fingerprint/:fingerprint", get(identity::handlers::get_by_fingerprint))
}

/// Spaces routes
fn spaces_routes() -> Router<Arc<AppState>> {
    Router::new()
        // List and search
        .route("/", get(spaces::handlers::list))
        .route("/", post(spaces::handlers::create))
        // By slug
        .route("/:slug", get(spaces::handlers::get_by_slug))
        .route("/:slug", patch(spaces::handlers::update))
        .route("/:slug", delete(spaces::handlers::delete))
        // Membership
        .route("/:slug/join", post(spaces::handlers::join))
        .route("/:slug/leave", post(spaces::handlers::leave))
        .route("/:slug/members", get(spaces::handlers::list_members))
        .route("/:slug/members/:identity_id", patch(spaces::handlers::update_member))
        .route("/:slug/members/:identity_id", delete(spaces::handlers::remove_member))
        // Posts in space
        .route("/:slug/posts", get(posts::handlers::list_by_space))
        .route("/:slug/posts", post(posts::handlers::create))
}

/// Posts routes
fn posts_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:id", get(posts::handlers::get_by_id))
        .route("/:id", patch(posts::handlers::update))
        .route("/:id", delete(posts::handlers::delete))
        // Voting
        .route("/:id/vote", post(votes::handlers::vote_post))
        .route("/:id/vote", delete(votes::handlers::unvote_post))
        // Comments
        .route("/:id/comments", get(comments::handlers::list_by_post))
        .route("/:id/comments", post(comments::handlers::create))
        // Moderation actions
        .route("/:id/pin", post(posts::handlers::pin))
        .route("/:id/unpin", post(posts::handlers::unpin))
        .route("/:id/lock", post(posts::handlers::lock))
        .route("/:id/unlock", post(posts::handlers::unlock))
}

/// Comments routes
fn comments_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:id", get(comments::handlers::get_by_id))
        .route("/:id", patch(comments::handlers::update))
        .route("/:id", delete(comments::handlers::delete))
        // Voting
        .route("/:id/vote", post(votes::handlers::vote_comment))
        .route("/:id/vote", delete(votes::handlers::unvote_comment))
        // Replies
        .route("/:id/replies", get(comments::handlers::list_replies))
}

/// Messages routes (E2E encrypted)
fn messages_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Conversations
        .route("/conversations", get(messages::handlers::list_conversations))
        .route("/conversations", post(messages::handlers::create_conversation))
        .route("/conversations/:id", get(messages::handlers::get_conversation))
        .route("/conversations/:id/messages", get(messages::handlers::list_messages))
        .route("/conversations/:id/messages", post(messages::handlers::send_message))
        .route("/conversations/:id/read", post(messages::handlers::mark_read))
        // Encryption key exchange
        .route("/keys/:identity_id", get(messages::handlers::get_public_key))
}

/// Media routes
fn media_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload", post(media::handlers::upload))
        .route("/:id", get(media::handlers::get_by_id))
        .route("/:id/download", get(media::handlers::download))
        .route("/:id/thumbnail", get(media::handlers::get_thumbnail))
}

/// Notifications routes
fn notifications_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(notifications::handlers::list))
        .route("/unread-count", get(notifications::handlers::unread_count))
        .route("/:id/read", post(notifications::handlers::mark_read))
        .route("/read-all", post(notifications::handlers::mark_all_read))
        // WebSocket for real-time notifications
        .route("/live", get(notifications::handlers::websocket_handler))
}

/// Feed routes
fn feed_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(feed::handlers::personalized_feed))
        .route("/all", get(feed::handlers::all_feed))
        .route("/popular", get(feed::handlers::popular_feed))
}

/// Moderation routes
fn moderation_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Reporting
        .route("/reports", post(moderation::handlers::create_report))
        .route("/reports", get(moderation::handlers::list_reports))
        .route("/reports/:id", get(moderation::handlers::get_report))
        .route("/reports/:id", patch(moderation::handlers::update_report))
        // Moderation actions
        .route("/posts/:id/remove", post(moderation::handlers::remove_post))
        .route("/comments/:id/remove", post(moderation::handlers::remove_comment))
        .route("/identities/:id/suspend", post(moderation::handlers::suspend_identity))
        .route("/identities/:id/unsuspend", post(moderation::handlers::unsuspend_identity))
}
