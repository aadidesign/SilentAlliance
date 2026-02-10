//! Vote handlers
//!
//! Handles upvoting/downvoting posts and comments with proper transaction
//! safety, vote value validation, and correct score calculations.

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// Allowed vote values: 1 (upvote), -1 (downvote), 0 (remove vote)
fn validate_vote_value(value: i16) -> ApiResult<()> {
    match value {
        -1 | 0 | 1 => Ok(()),
        _ => Err(ApiError::InvalidInput(
            "Vote value must be -1, 0, or 1".to_string(),
        )),
    }
}

/// Vote on a post
pub async fn vote_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<VoteRequest>,
) -> ApiResult<StatusCode> {
    validate_vote_value(request.value)?;

    let new_value = request.value;

    // Use a transaction to ensure atomic vote + score update
    let mut tx = state.db.pool().begin().await?;

    // Check post exists (lock row for update to prevent races)
    let post = sqlx::query!(
        "SELECT id, author_id FROM posts WHERE id = $1 AND is_removed = false FOR UPDATE",
        post_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    // Get existing vote
    let existing = sqlx::query!(
        "SELECT id, vote_value FROM votes WHERE identity_id = $1 AND target_type = 'post' AND target_id = $2 FOR UPDATE",
        user.identity_id,
        post_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let old_value = existing.as_ref().map(|v| v.vote_value).unwrap_or(0);

    // Short-circuit if vote hasn't changed
    if old_value == new_value {
        tx.commit().await?;
        return Ok(StatusCode::OK);
    }

    if new_value == 0 {
        // Remove vote
        if let Some(v) = existing {
            sqlx::query!("DELETE FROM votes WHERE id = $1", v.id)
                .execute(&mut *tx)
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
        .execute(&mut *tx)
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
        .execute(&mut *tx)
        .await?;
    }

    // Calculate correct upvote/downvote delta
    // old_value -> new_value: adjust upvotes and downvotes independently
    let upvote_delta = compute_delta(old_value, new_value, 1);
    let downvote_delta = compute_delta(old_value, new_value, -1);

    sqlx::query!(
        r#"
        UPDATE posts
        SET upvotes = GREATEST(0, upvotes + $2),
            downvotes = GREATEST(0, downvotes + $3),
            score = (upvotes + $2) - (downvotes + $3)
        WHERE id = $1
        "#,
        post_id,
        upvote_delta as i32,
        downvote_delta as i32
    )
    .execute(&mut *tx)
    .await?;

    // Update author karma
    let karma_delta = (new_value - old_value) as i32;
    if karma_delta != 0 {
        if let Some(author_id) = post.author_id {
            sqlx::query!(
                "UPDATE identities SET karma = karma + $1 WHERE id = $2",
                karma_delta,
                author_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
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
    validate_vote_value(request.value)?;

    let new_value = request.value;

    let mut tx = state.db.pool().begin().await?;

    let comment = sqlx::query!(
        "SELECT id, author_id FROM comments WHERE id = $1 AND is_removed = false FOR UPDATE",
        comment_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    let existing = sqlx::query!(
        "SELECT id, vote_value FROM votes WHERE identity_id = $1 AND target_type = 'comment' AND target_id = $2 FOR UPDATE",
        user.identity_id,
        comment_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let old_value = existing.as_ref().map(|v| v.vote_value).unwrap_or(0);

    if old_value == new_value {
        tx.commit().await?;
        return Ok(StatusCode::OK);
    }

    if new_value == 0 {
        if let Some(v) = existing {
            sqlx::query!("DELETE FROM votes WHERE id = $1", v.id)
                .execute(&mut *tx)
                .await?;
        }
    } else if existing.is_some() {
        sqlx::query!(
            "UPDATE votes SET vote_value = $1 WHERE identity_id = $2 AND target_type = 'comment' AND target_id = $3",
            new_value,
            user.identity_id,
            comment_id
        )
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO votes (id, identity_id, target_type, target_id, vote_value, created_at) VALUES ($1, $2, 'comment', $3, $4, NOW())",
            Uuid::new_v4(),
            user.identity_id,
            comment_id,
            new_value
        )
        .execute(&mut *tx)
        .await?;
    }

    // Calculate correct deltas
    let upvote_delta = compute_delta(old_value, new_value, 1);
    let downvote_delta = compute_delta(old_value, new_value, -1);

    sqlx::query!(
        r#"
        UPDATE comments
        SET upvotes = GREATEST(0, upvotes + $2),
            downvotes = GREATEST(0, downvotes + $3),
            score = (upvotes + $2) - (downvotes + $3)
        WHERE id = $1
        "#,
        comment_id,
        upvote_delta as i32,
        downvote_delta as i32
    )
    .execute(&mut *tx)
    .await?;

    let karma_delta = (new_value - old_value) as i32;
    if karma_delta != 0 {
        if let Some(author_id) = comment.author_id {
            sqlx::query!(
                "UPDATE identities SET karma = karma + $1 WHERE id = $2",
                karma_delta,
                author_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
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

/// Compute the delta for a specific vote direction.
/// `direction` is 1 for upvotes, -1 for downvotes.
///
/// Returns +1 if gaining this direction, -1 if losing it, 0 if unchanged.
fn compute_delta(old_value: i16, new_value: i16, direction: i16) -> i16 {
    let was_this = if old_value == direction { 1i16 } else { 0 };
    let is_this = if new_value == direction { 1i16 } else { 0 };
    is_this - was_this
}
