//! Comments handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::{AuthenticatedUser, OptionalUser, check_moderator};
use crate::AppState;

/// Maximum comment nesting depth to prevent abuse
const MAX_COMMENT_DEPTH: i32 = 10;

/// List comments for a post
pub async fn list_by_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Comment>>> {
    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT id, post_id, parent_id, author_id, content, depth, path,
               upvotes, downvotes, score, is_removed, removed_reason,
               created_at, updated_at
        FROM comments
        WHERE post_id = $1 AND is_removed = false
        ORDER BY path, created_at
        LIMIT $2 OFFSET $3
        "#,
        post_id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM comments WHERE post_id = $1 AND is_removed = false",
        post_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: comments,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Create a comment
pub async fn create(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<CreateCommentRequest>,
) -> ApiResult<(StatusCode, Json<Comment>)> {
    request.validate()?;

    // Check post exists and is not locked
    let post = sqlx::query!("SELECT is_locked, is_removed FROM posts WHERE id = $1", post_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if post.is_locked {
        return Err(ApiError::OperationNotAllowed("Post is locked".to_string()));
    }

    if post.is_removed {
        return Err(ApiError::Gone);
    }

    // Calculate depth and path
    let (depth, path) = if let Some(parent_id) = request.parent_id {
        let parent = sqlx::query!(
            "SELECT depth, path, post_id FROM comments WHERE id = $1",
            parent_id
        )
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Parent comment not found".to_string()))?;

        // Verify parent comment belongs to the same post
        if parent.post_id != post_id {
            return Err(ApiError::InvalidInput(
                "Parent comment does not belong to this post".to_string(),
            ));
        }

        let new_depth = parent.depth + 1;
        if new_depth > MAX_COMMENT_DEPTH {
            return Err(ApiError::OperationNotAllowed(format!(
                "Maximum comment depth of {} exceeded",
                MAX_COMMENT_DEPTH
            )));
        }

        (new_depth, format!("{}.{}", parent.path, parent_id))
    } else {
        (0, "".to_string())
    };

    let id = Uuid::new_v4();
    let full_path = if path.is_empty() { id.to_string() } else { format!("{}.{}", path, id) };

    let comment = sqlx::query_as!(
        Comment,
        r#"
        INSERT INTO comments (id, post_id, parent_id, author_id, content, depth, path, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, post_id, parent_id, author_id, content, depth, path,
                  upvotes, downvotes, score, is_removed, removed_reason,
                  created_at, updated_at
        "#,
        id,
        post_id,
        request.parent_id,
        user.identity_id,
        request.content,
        depth,
        full_path
    )
    .fetch_one(state.db.pool())
    .await?;

    // Increment comment count
    sqlx::query!("UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1", post_id)
        .execute(state.db.pool())
        .await?;

    Ok((StatusCode::CREATED, Json(comment)))
}

/// Get comment by ID
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Comment>> {
    let comment = sqlx::query_as!(
        Comment,
        r#"
        SELECT id, post_id, parent_id, author_id, content, depth, path,
               upvotes, downvotes, score, is_removed, removed_reason,
               created_at, updated_at
        FROM comments
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    Ok(Json(comment))
}

/// Update a comment
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<UpdateCommentRequest>,
) -> ApiResult<Json<Comment>> {
    request.validate()?;

    let comment = sqlx::query!("SELECT author_id FROM comments WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    if comment.author_id != Some(user.identity_id) {
        return Err(ApiError::Forbidden);
    }

    let updated = sqlx::query_as!(
        Comment,
        r#"
        UPDATE comments SET content = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, post_id, parent_id, author_id, content, depth, path,
                  upvotes, downvotes, score, is_removed, removed_reason,
                  created_at, updated_at
        "#,
        id,
        request.content
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(updated))
}

/// Delete a comment (soft delete) — allowed for the author or a space moderator
pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let comment = sqlx::query!(
        "SELECT author_id, post_id FROM comments WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    let is_author = comment.author_id == Some(user.identity_id);

    // If not the author, check if user is a moderator of the post's space
    if !is_author {
        let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", comment.post_id)
            .fetch_optional(state.db.pool())
            .await?
            .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

        if !check_moderator(&state, user.identity_id, post.space_id).await? {
            return Err(ApiError::Forbidden);
        }
    }

    let reason = if is_author {
        "Deleted by author"
    } else {
        "Removed by moderator"
    };

    sqlx::query!(
        "UPDATE comments SET is_removed = true, removed_reason = $2 WHERE id = $1",
        id,
        reason
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List replies to a comment
pub async fn list_replies(
    State(state): State<Arc<AppState>>,
    Path(parent_id): Path<Uuid>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Comment>>> {
    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT id, post_id, parent_id, author_id, content, depth, path,
               upvotes, downvotes, score, is_removed, removed_reason,
               created_at, updated_at
        FROM comments
        WHERE parent_id = $1 AND is_removed = false
        ORDER BY score DESC, created_at
        LIMIT $2 OFFSET $3
        "#,
        parent_id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM comments WHERE parent_id = $1 AND is_removed = false",
        parent_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: comments,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}
