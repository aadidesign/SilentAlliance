//! Vote handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// Vote on a post
pub async fn vote_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<VoteRequest>,
) -> ApiResult<StatusCode> {
    // Check post exists
    let post = sqlx::query!("SELECT id, author_id FROM posts WHERE id = $1 AND is_removed = false", post_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    // Get existing vote
    let existing = sqlx::query!(
        "SELECT id, vote_value FROM votes WHERE identity_id = $1 AND target_type = 'post' AND target_id = $2",
        user.identity_id,
        post_id
    )
    .fetch_optional(state.db.pool())
    .await?;

    let old_value = existing.as_ref().map(|v| v.vote_value).unwrap_or(0);
    let new_value = request.value;

    if new_value == 0 {
        // Remove vote
        if let Some(v) = existing {
            sqlx::query!("DELETE FROM votes WHERE id = $1", v.id)
                .execute(state.db.pool())
                .await?;
        }
    } else if existing.is_some() {
        // Update vote
        sqlx::query!(
            "UPDATE votes SET vote_value = $1 WHERE identity_id = $2 AND target_type = 'post' AND target_id = $3",
            new_value,
            user.identity_id,
            post_id
        )
        .execute(state.db.pool())
        .await?;
    } else {
        // Create vote
        sqlx::query!(
            "INSERT INTO votes (id, identity_id, target_type, target_id, vote_value, created_at) VALUES ($1, $2, 'post', $3, $4, NOW())",
            Uuid::new_v4(),
            user.identity_id,
            post_id,
            new_value
        )
        .execute(state.db.pool())
        .await?;
    }

    // Update post vote counts
    let vote_delta = new_value - old_value;
    if vote_delta != 0 {
        if vote_delta > 0 {
            sqlx::query!("UPDATE posts SET upvotes = upvotes + 1, score = upvotes - downvotes + 1 WHERE id = $1", post_id)
                .execute(state.db.pool())
                .await?;
        } else {
            sqlx::query!("UPDATE posts SET downvotes = downvotes + 1, score = upvotes - downvotes - 1 WHERE id = $1", post_id)
                .execute(state.db.pool())
                .await?;
        }

        // Update author karma
        if let Some(author_id) = post.author_id {
            sqlx::query!("UPDATE identities SET karma = karma + $1 WHERE id = $2", vote_delta as i32, author_id)
                .execute(state.db.pool())
                .await?;
        }
    }

    Ok(StatusCode::OK)
}

/// Remove vote from a post
pub async fn unvote_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    vote_post(State(state), Path(post_id), user, Json(VoteRequest { value: 0 })).await
}

/// Vote on a comment
pub async fn vote_comment(
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<VoteRequest>,
) -> ApiResult<StatusCode> {
    let comment = sqlx::query!("SELECT id, author_id FROM comments WHERE id = $1 AND is_removed = false", comment_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    let existing = sqlx::query!(
        "SELECT id, vote_value FROM votes WHERE identity_id = $1 AND target_type = 'comment' AND target_id = $2",
        user.identity_id,
        comment_id
    )
    .fetch_optional(state.db.pool())
    .await?;

    let old_value = existing.as_ref().map(|v| v.vote_value).unwrap_or(0);
    let new_value = request.value;

    if new_value == 0 {
        if let Some(v) = existing {
            sqlx::query!("DELETE FROM votes WHERE id = $1", v.id)
                .execute(state.db.pool())
                .await?;
        }
    } else if existing.is_some() {
        sqlx::query!(
            "UPDATE votes SET vote_value = $1 WHERE identity_id = $2 AND target_type = 'comment' AND target_id = $3",
            new_value,
            user.identity_id,
            comment_id
        )
        .execute(state.db.pool())
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO votes (id, identity_id, target_type, target_id, vote_value, created_at) VALUES ($1, $2, 'comment', $3, $4, NOW())",
            Uuid::new_v4(),
            user.identity_id,
            comment_id,
            new_value
        )
        .execute(state.db.pool())
        .await?;
    }

    // Update comment scores and author karma
    let vote_delta = new_value - old_value;
    if vote_delta != 0 {
        sqlx::query!(
            "UPDATE comments SET upvotes = upvotes + CASE WHEN $1 > 0 THEN 1 ELSE 0 END, downvotes = downvotes + CASE WHEN $1 < 0 THEN 1 ELSE 0 END, score = score + $1 WHERE id = $2",
            vote_delta as i32,
            comment_id
        )
        .execute(state.db.pool())
        .await?;

        if let Some(author_id) = comment.author_id {
            sqlx::query!("UPDATE identities SET karma = karma + $1 WHERE id = $2", vote_delta as i32, author_id)
                .execute(state.db.pool())
                .await?;
        }
    }

    Ok(StatusCode::OK)
}

/// Remove vote from a comment
pub async fn unvote_comment(
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    vote_comment(State(state), Path(comment_id), user, Json(VoteRequest { value: 0 })).await
}
