//! Spaces (communities) handlers

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
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::{AuthenticatedUser, check_space_admin};
use crate::AppState;

/// List spaces with optional search
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SpaceListParams>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Space>>> {
    let spaces = if let Some(search) = &params.search {
        sqlx::query_as!(
            Space,
            r#"
            SELECT id, name, slug, description, rules, icon_url, banner_url,
                   is_private, is_nsfw, creator_id, subscriber_count, post_count,
                   created_at, updated_at
            FROM spaces
            WHERE (name ILIKE $1 OR description ILIKE $1) AND is_private = false
            ORDER BY subscriber_count DESC
            LIMIT $2 OFFSET $3
            "#,
            format!("%{}%", search),
            pagination.limit,
            pagination.offset
        )
        .fetch_all(state.db.pool())
        .await?
    } else {
        sqlx::query_as!(
            Space,
            r#"
            SELECT id, name, slug, description, rules, icon_url, banner_url,
                   is_private, is_nsfw, creator_id, subscriber_count, post_count,
                   created_at, updated_at
            FROM spaces
            WHERE is_private = false
            ORDER BY subscriber_count DESC
            LIMIT $1 OFFSET $2
            "#,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(state.db.pool())
        .await?
    };

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM spaces WHERE is_private = false")
        .fetch_one(state.db.pool())
        .await?
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: spaces,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Create a new space
pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateSpaceRequest>,
) -> ApiResult<(StatusCode, Json<Space>)> {
    request.validate()?;

    // Check karma requirement
    let karma: i32 = sqlx::query_scalar!("SELECT karma FROM identities WHERE id = $1", user.identity_id)
        .fetch_one(state.db.pool())
        .await?
        .unwrap_or(0);

    if karma < 100 {
        return Err(ApiError::OperationNotAllowed(
            "Need at least 100 karma to create a space".to_string(),
        ));
    }

    // Generate a URL-safe slug from the name:
    // 1. Lowercase everything
    // 2. Replace spaces/underscores with hyphens
    // 3. Strip all non-alphanumeric/hyphen characters
    // 4. Collapse consecutive hyphens
    // 5. Trim leading/trailing hyphens
    let slug: String = request
        .name
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' || c == '_' { '-' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    // Collapse consecutive hyphens and trim edges
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        return Err(ApiError::InvalidInput(
            "Space name must contain at least one alphanumeric character".to_string(),
        ));
    }

    // Check if slug is taken
    let existing = sqlx::query_scalar!("SELECT id FROM spaces WHERE slug = $1", &slug)
        .fetch_optional(state.db.pool())
        .await?;

    if existing.is_some() {
        return Err(ApiError::Conflict("Space name already taken".to_string()));
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let space = sqlx::query_as!(
        Space,
        r#"
        INSERT INTO spaces (id, name, slug, description, is_private, is_nsfw, creator_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
        RETURNING id, name, slug, description, rules, icon_url, banner_url,
                  is_private, is_nsfw, creator_id, subscriber_count, post_count,
                  created_at, updated_at
        "#,
        id,
        request.name,
        slug,
        request.description,
        request.is_private.unwrap_or(false),
        request.is_nsfw.unwrap_or(false),
        user.identity_id,
        now
    )
    .fetch_one(state.db.pool())
    .await?;

    // Add creator as admin
    sqlx::query!(
        "INSERT INTO space_members (id, space_id, identity_id, role, joined_at) VALUES ($1, $2, $3, 'admin', $4)",
        Uuid::new_v4(),
        id,
        user.identity_id,
        now
    )
    .execute(state.db.pool())
    .await?;

    Ok((StatusCode::CREATED, Json(space)))
}

/// Get space by slug
pub async fn get_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Space>> {
    let space = sqlx::query_as!(
        Space,
        r#"
        SELECT id, name, slug, description, rules, icon_url, banner_url,
               is_private, is_nsfw, creator_id, subscriber_count, post_count,
               created_at, updated_at
        FROM spaces
        WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    Ok(Json(space))
}

/// Update space
pub async fn update(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
    Json(request): Json<UpdateSpaceRequest>,
) -> ApiResult<Json<Space>> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    // Check admin permission
    if !check_space_admin(&state, user.identity_id, space.id).await? {
        return Err(ApiError::Forbidden);
    }

    let updated = sqlx::query_as!(
        Space,
        r#"
        UPDATE spaces
        SET description = COALESCE($2, description),
            is_private = COALESCE($3, is_private),
            is_nsfw = COALESCE($4, is_nsfw),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, slug, description, rules, icon_url, banner_url,
                  is_private, is_nsfw, creator_id, subscriber_count, post_count,
                  created_at, updated_at
        "#,
        space.id,
        request.description,
        request.is_private,
        request.is_nsfw
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(updated))
}

/// Delete space
pub async fn delete(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    let space = sqlx::query!("SELECT id, creator_id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    // Only creator can delete
    if space.creator_id != Some(user.identity_id) {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!("DELETE FROM spaces WHERE id = $1", space.id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Join a space
pub async fn join(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    let space = sqlx::query!("SELECT id, is_private FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    if space.is_private {
        return Err(ApiError::Forbidden);
    }

    // Check if already member
    let existing = sqlx::query!(
        "SELECT id FROM space_members WHERE space_id = $1 AND identity_id = $2",
        space.id,
        user.identity_id
    )
    .fetch_optional(state.db.pool())
    .await?;

    if existing.is_some() {
        return Ok(StatusCode::OK);
    }

    sqlx::query!(
        "INSERT INTO space_members (id, space_id, identity_id, role, joined_at) VALUES ($1, $2, $3, 'member', NOW())",
        Uuid::new_v4(),
        space.id,
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    // Increment subscriber count
    sqlx::query!("UPDATE spaces SET subscriber_count = subscriber_count + 1 WHERE id = $1", space.id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::CREATED)
}

/// Leave a space
pub async fn leave(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    let result = sqlx::query!(
        "DELETE FROM space_members WHERE space_id = $1 AND identity_id = $2",
        space.id,
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    if result.rows_affected() > 0 {
        sqlx::query!("UPDATE spaces SET subscriber_count = subscriber_count - 1 WHERE id = $1", space.id)
            .execute(state.db.pool())
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// List space members
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<SpaceMemberWithIdentity>>> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    let members = sqlx::query_as!(
        SpaceMemberWithIdentity,
        r#"
        SELECT sm.id, sm.space_id, sm.identity_id, sm.role as "role: MemberRole", sm.joined_at,
               i.display_name, i.public_key_fingerprint
        FROM space_members sm
        JOIN identities i ON i.id = sm.identity_id
        WHERE sm.space_id = $1
        ORDER BY sm.joined_at DESC
        LIMIT $2 OFFSET $3
        "#,
        space.id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM space_members WHERE space_id = $1", space.id)
        .fetch_one(state.db.pool())
        .await?
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: members,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Update member role
pub async fn update_member(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path((slug, identity_id)): Path<(String, Uuid)>,
    Json(request): Json<UpdateMemberRequest>,
) -> ApiResult<StatusCode> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    if !check_space_admin(&state, user.identity_id, space.id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!(
        "UPDATE space_members SET role = $1 WHERE space_id = $2 AND identity_id = $3",
        request.role.to_string(),
        space.id,
        identity_id
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

/// Remove member from space
pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path((slug, identity_id)): Path<(String, Uuid)>,
) -> ApiResult<StatusCode> {
    let space = sqlx::query!("SELECT id FROM spaces WHERE slug = $1", slug)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Space not found".to_string()))?;

    if !check_space_admin(&state, user.identity_id, space.id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!(
        "DELETE FROM space_members WHERE space_id = $1 AND identity_id = $2",
        space.id,
        identity_id
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// Request/Response types

#[derive(Debug, serde::Deserialize)]
pub struct SpaceListParams {
    pub search: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SpaceMemberWithIdentity {
    pub id: Uuid,
    pub space_id: Uuid,
    pub identity_id: Uuid,
    pub role: MemberRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub display_name: Option<String>,
    pub public_key_fingerprint: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateMemberRequest {
    pub role: MemberRole,
}
