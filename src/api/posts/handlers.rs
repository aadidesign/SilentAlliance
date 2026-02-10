//! Posts handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::domain::services::feed::calculate_hot_score;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::{AuthenticatedUser, OptionalUser, check_moderator};
use crate::AppState;

/// List posts in a space
pub async fn list_by_space(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Query(params): Query<FeedParams>,
    user: OptionalUser,
) -> ApiResult<Json<PaginatedResponse<PostWithContext>>> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, space_id, author_id, title, content,
               content_type as "content_type: ContentType",
               url, media_ids, upvotes, downvotes, score, comment_count,
               is_pinned, is_locked, is_removed, removed_reason,
               created_at, updated_at
        FROM posts
        WHERE space_id = $1 AND is_removed = false
        ORDER BY is_pinned DESC, score DESC, created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        space.id,
        params.pagination.limit,
        params.pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    // Get user votes if authenticated
    let user_votes = if let Some(ref u) = user.0 {
        get_user_votes_for_posts(&state, u.identity_id, &posts).await?
    } else {
        std::collections::HashMap::new()
    };

    // Build response with context
    let posts_with_context: Vec<PostWithContext> = posts
        .into_iter()
        .map(|post| {
            let user_vote = user_votes.get(&post.id).copied();
            PostWithContext {
                post,
                author: None, // Would load from cache/batch
                space: None,
                user_vote,
            }
        })
        .collect();

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM posts WHERE space_id = $1 AND is_removed = false",
        space.id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: posts_with_context,
        pagination: PaginationInfo::new(total, params.pagination.limit, params.pagination.offset),
    }))
}

/// Create a new post
pub async fn create(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreatePostRequest>,
) -> ApiResult<(StatusCode, Json<Post>)> {
    request.validate()?;

    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    // Check if user is member
    let is_member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM space_members WHERE space_id = $1 AND identity_id = $2)",
        space.id,
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(false);

    if !is_member {
        return Err(ApiError::Forbidden);
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let content_type = request.content_type.unwrap_or_default();

    let post = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (id, space_id, author_id, title, content, content_type, url, media_ids, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        RETURNING id, space_id, author_id, title, content,
                  content_type as "content_type: ContentType",
                  url, media_ids, upvotes, downvotes, score, comment_count,
                  is_pinned, is_locked, is_removed, removed_reason,
                  created_at, updated_at
        "#,
        id,
        space.id,
        user.identity_id,
        request.title,
        request.content,
        content_type.to_string(),
        request.url,
        &request.media_ids.unwrap_or_default() as &[Uuid],
        now
    )
    .fetch_one(state.db.pool())
    .await?;

    // Increment space post count
    sqlx::query!("UPDATE spaces SET post_count = post_count + 1 WHERE id = $1", space.id)
        .execute(state.db.pool())
        .await?;

    // Award karma for posting
    sqlx::query!("UPDATE identities SET karma = karma + 1 WHERE id = $1", user.identity_id)
        .execute(state.db.pool())
        .await?;

    Ok((StatusCode::CREATED, Json(post)))
}

/// Get post by ID
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: OptionalUser,
) -> ApiResult<Json<PostWithContext>> {
    let post = sqlx::query_as!(
        Post,
        r#"
        SELECT id, space_id, author_id, title, content,
               content_type as "content_type: ContentType",
               url, media_ids, upvotes, downvotes, score, comment_count,
               is_pinned, is_locked, is_removed, removed_reason,
               created_at, updated_at
        FROM posts
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if post.is_removed {
        return Err(ApiError::Gone);
    }

    let user_vote = if let Some(ref u) = user.0 {
        sqlx::query_scalar!(
            "SELECT vote_value FROM votes WHERE identity_id = $1 AND target_type = 'post' AND target_id = $2",
            u.identity_id,
            id
        )
        .fetch_optional(state.db.pool())
        .await?
        .flatten()
    } else {
        None
    };

    Ok(Json(PostWithContext {
        post,
        author: None,
        space: None,
        user_vote,
    }))
}

/// Update a post
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<UpdatePostRequest>,
) -> ApiResult<Json<Post>> {
    request.validate()?;

    let post = sqlx::query!(
        "SELECT author_id, is_locked, is_removed FROM posts WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if post.is_removed {
        return Err(ApiError::Gone);
    }

    if post.is_locked {
        return Err(ApiError::OperationNotAllowed("Post is locked".to_string()));
    }

    if post.author_id != Some(user.identity_id) {
        return Err(ApiError::Forbidden);
    }

    let updated = sqlx::query_as!(
        Post,
        r#"
        UPDATE posts
        SET content = COALESCE($2, content), updated_at = NOW()
        WHERE id = $1
        RETURNING id, space_id, author_id, title, content,
                  content_type as "content_type: ContentType",
                  url, media_ids, upvotes, downvotes, score, comment_count,
                  is_pinned, is_locked, is_removed, removed_reason,
                  created_at, updated_at
        "#,
        id,
        request.content
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(updated))
}

/// Delete a post (soft delete)
pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let post = sqlx::query!("SELECT author_id, space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    // Check if author or moderator
    let is_author = post.author_id == Some(user.identity_id);
    let is_mod = check_moderator(&state, user.identity_id, post.space_id).await?;

    if !is_author && !is_mod {
        return Err(ApiError::Forbidden);
    }

    let reason = if is_author {
        "Deleted by author"
    } else {
        "Removed by moderator"
    };

    sqlx::query!(
        "UPDATE posts SET is_removed = true, removed_reason = $2 WHERE id = $1",
        id,
        reason
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Pin a post
pub async fn pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("UPDATE posts SET is_pinned = true WHERE id = $1", id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::OK)
}

/// Unpin a post
pub async fn unpin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("UPDATE posts SET is_pinned = false WHERE id = $1", id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::OK)
}

/// Lock a post (prevent new comments)
pub async fn lock(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("UPDATE posts SET is_locked = true WHERE id = $1", id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::OK)
}

/// Unlock a post
pub async fn unlock(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("UPDATE posts SET is_locked = false WHERE id = $1", id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::OK)
}

// Helper functions

async fn get_user_votes_for_posts(
    state: &Arc<AppState>,
    identity_id: Uuid,
    posts: &[Post],
) -> ApiResult<std::collections::HashMap<Uuid, i16>> {
    let post_ids: Vec<Uuid> = posts.iter().map(|p| p.id).collect();

    let votes = sqlx::query!(
        "SELECT target_id, vote_value FROM votes WHERE identity_id = $1 AND target_type = 'post' AND target_id = ANY($2)",
        identity_id,
        &post_ids
    )
    .fetch_all(state.db.pool())
    .await?;

    Ok(votes.into_iter().map(|v| (v.target_id, v.vote_value)).collect())
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ContentType::Text => "text",
            ContentType::Link => "link",
            ContentType::Media => "media",
            ContentType::Poll => "poll",
        };
        write!(f, "{}", s)
    }
}
