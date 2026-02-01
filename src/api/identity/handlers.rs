//! Identity handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// Get current authenticated identity
pub async fn get_current(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> ApiResult<Json<Identity>> {
    let identity = sqlx::query_as!(
        Identity,
        r#"
        SELECT id, public_key, public_key_fingerprint, display_name, avatar_hash,
               bio, karma, is_verified, is_suspended, suspended_reason, suspended_until,
               created_at, updated_at
        FROM identities
        WHERE id = $1
        "#,
        user.identity_id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Identity not found".to_string()))?;

    Ok(Json(identity))
}

/// Update current identity profile
pub async fn update_current(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(request): Json<UpdateIdentityRequest>,
) -> ApiResult<Json<Identity>> {
    request.validate()?;

    let identity = sqlx::query_as!(
        Identity,
        r#"
        UPDATE identities
        SET display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, public_key, public_key_fingerprint, display_name, avatar_hash,
                  bio, karma, is_verified, is_suspended, suspended_reason, suspended_until,
                  created_at, updated_at
        "#,
        user.identity_id,
        request.display_name,
        request.bio
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(identity))
}

/// Get identity by ID
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<IdentityPublic>> {
    let identity = sqlx::query_as!(
        Identity,
        r#"
        SELECT id, public_key, public_key_fingerprint, display_name, avatar_hash,
               bio, karma, is_verified, is_suspended, suspended_reason, suspended_until,
               created_at, updated_at
        FROM identities
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Identity not found".to_string()))?;

    Ok(Json(identity.into()))
}

/// Get identity by fingerprint
pub async fn get_by_fingerprint(
    State(state): State<Arc<AppState>>,
    Path(fingerprint): Path<String>,
) -> ApiResult<Json<IdentityPublic>> {
    let identity = sqlx::query_as!(
        Identity,
        r#"
        SELECT id, public_key, public_key_fingerprint, display_name, avatar_hash,
               bio, karma, is_verified, is_suspended, suspended_reason, suspended_until,
               created_at, updated_at
        FROM identities
        WHERE public_key_fingerprint = $1
        "#,
        fingerprint
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Identity not found".to_string()))?;

    Ok(Json(identity.into()))
}

/// Get posts by identity
pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Post>>> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, space_id, author_id, title, content,
               content_type as "content_type: ContentType",
               url, media_ids, upvotes, downvotes, score, comment_count,
               is_pinned, is_locked, is_removed, removed_reason,
               created_at, updated_at
        FROM posts
        WHERE author_id = $1 AND is_removed = false
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM posts WHERE author_id = $1 AND is_removed = false",
        id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Get comments by identity
pub async fn get_comments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Comment>>> {
    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT id, post_id, parent_id, author_id, content, depth, path,
               upvotes, downvotes, score, is_removed, removed_reason,
               created_at, updated_at
        FROM comments
        WHERE author_id = $1 AND is_removed = false
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM comments WHERE author_id = $1 AND is_removed = false",
        id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: comments,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// List active sessions
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> ApiResult<Json<Vec<SessionInfo>>> {
    let sessions = sqlx::query_as!(
        SessionInfo,
        r#"
        SELECT id, created_at, expires_at, revoked
        FROM refresh_tokens
        WHERE identity_id = $1 AND revoked = false AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
        user.identity_id
    )
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(sessions))
}

/// Revoke a specific session
pub async fn revoke_session(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query!(
        "UPDATE refresh_tokens SET revoked = true WHERE id = $1 AND identity_id = $2",
        session_id,
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Session not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Session info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SessionInfo {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked: bool,
}
