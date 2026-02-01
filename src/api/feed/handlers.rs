//! Feed handlers with ranking algorithms

use axum::{extract::{Query, State}, Json};
use std::sync::Arc;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::ApiResult;
use crate::middleware::auth::{AuthenticatedUser, OptionalUser};
use crate::AppState;

/// Get personalized feed (posts from subscribed spaces)
pub async fn personalized_feed(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Query(params): Query<FeedParams>,
) -> ApiResult<Json<PaginatedResponse<PostWithContext>>> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.space_id, p.author_id, p.title, p.content,
               p.content_type as "content_type: ContentType",
               p.url, p.media_ids, p.upvotes, p.downvotes, p.score, p.comment_count,
               p.is_pinned, p.is_locked, p.is_removed, p.removed_reason,
               p.created_at, p.updated_at
        FROM posts p
        JOIN space_members sm ON sm.space_id = p.space_id
        WHERE sm.identity_id = $1 AND p.is_removed = false
        ORDER BY p.score DESC, p.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.identity_id,
        params.pagination.limit,
        params.pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let posts_with_context: Vec<PostWithContext> = posts
        .into_iter()
        .map(|post| PostWithContext {
            post,
            author: None,
            space: None,
            user_vote: None,
        })
        .collect();

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM posts p
        JOIN space_members sm ON sm.space_id = p.space_id
        WHERE sm.identity_id = $1 AND p.is_removed = false
        "#,
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts_with_context,
        pagination: PaginationInfo::new(total, params.pagination.limit, params.pagination.offset),
    }))
}

/// Get all posts across all public spaces
pub async fn all_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeedParams>,
    _user: OptionalUser,
) -> ApiResult<Json<PaginatedResponse<PostWithContext>>> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.space_id, p.author_id, p.title, p.content,
               p.content_type as "content_type: ContentType",
               p.url, p.media_ids, p.upvotes, p.downvotes, p.score, p.comment_count,
               p.is_pinned, p.is_locked, p.is_removed, p.removed_reason,
               p.created_at, p.updated_at
        FROM posts p
        JOIN spaces s ON s.id = p.space_id
        WHERE p.is_removed = false AND s.is_private = false
        ORDER BY p.score DESC, p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        params.pagination.limit,
        params.pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let posts_with_context: Vec<PostWithContext> = posts
        .into_iter()
        .map(|post| PostWithContext {
            post,
            author: None,
            space: None,
            user_vote: None,
        })
        .collect();

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM posts p JOIN spaces s ON s.id = p.space_id WHERE p.is_removed = false AND s.is_private = false"
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts_with_context,
        pagination: PaginationInfo::new(total, params.pagination.limit, params.pagination.offset),
    }))
}

/// Get popular feed (trending posts)
pub async fn popular_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeedParams>,
    _user: OptionalUser,
) -> ApiResult<Json<PaginatedResponse<PostWithContext>>> {
    // Popular = high score + recent activity
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.space_id, p.author_id, p.title, p.content,
               p.content_type as "content_type: ContentType",
               p.url, p.media_ids, p.upvotes, p.downvotes, p.score, p.comment_count,
               p.is_pinned, p.is_locked, p.is_removed, p.removed_reason,
               p.created_at, p.updated_at
        FROM posts p
        JOIN spaces s ON s.id = p.space_id
        WHERE p.is_removed = false AND s.is_private = false
              AND p.created_at > NOW() - INTERVAL '7 days'
        ORDER BY (p.upvotes + p.comment_count) DESC, p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        params.pagination.limit,
        params.pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let posts_with_context: Vec<PostWithContext> = posts
        .into_iter()
        .map(|post| PostWithContext {
            post,
            author: None,
            space: None,
            user_vote: None,
        })
        .collect();

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM posts p
        JOIN spaces s ON s.id = p.space_id
        WHERE p.is_removed = false AND s.is_private = false
              AND p.created_at > NOW() - INTERVAL '7 days'
        "#
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts_with_context,
        pagination: PaginationInfo::new(total, params.pagination.limit, params.pagination.offset),
    }))
}
