//! Content moderation handlers

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::{AuthenticatedUser, check_moderator};
use crate::AppState;

/// Check if user is a global moderator (has moderator role in any space,
/// or is an admin). For production, this should be replaced with a
/// dedicated global_roles table or admin flag on the identity.
async fn require_global_moderator(
    state: &Arc<AppState>,
    identity_id: Uuid,
) -> ApiResult<()> {
    let is_mod: Option<bool> = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM space_members
            WHERE identity_id = $1 AND role IN ('moderator', 'admin')
        ) as "exists!"
        "#,
        identity_id
    )
    .fetch_one(state.db.pool())
    .await?;

    if !is_mod.unwrap_or(false) {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

/// Create a report
pub async fn create_report(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateReportRequest>,
) -> ApiResult<(StatusCode, Json<Report>)> {
    request.validate()?;

    let id = Uuid::new_v4();
    let report = sqlx::query_as!(
        Report,
        r#"
        INSERT INTO reports (id, reporter_id, target_type, target_id, reason, description, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW())
        RETURNING id, reporter_id, target_type as "target_type: ReportTargetType",
                  target_id, reason as "reason: ReportReason", description,
                  status as "status: ReportStatus", reviewed_by, review_notes, created_at, reviewed_at
        "#,
        id,
        user.identity_id,
        request.target_type.to_string(),
        request.target_id,
        request.reason.to_string(),
        request.description
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok((StatusCode::CREATED, Json(report)))
}

/// List reports (moderators only)
pub async fn list_reports(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Query(params): Query<ReportListParams>,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Report>>> {
    // Verify the user has moderator privileges
    require_global_moderator(&state, user.identity_id).await?;

    let reports = sqlx::query_as!(
        Report,
        r#"
        SELECT id, reporter_id, target_type as "target_type: ReportTargetType",
               target_id, reason as "reason: ReportReason", description,
               status as "status: ReportStatus", reviewed_by, review_notes, created_at, reviewed_at
        FROM reports
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        params.status.map(|s| s.to_string()),
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM reports")
        .fetch_one(state.db.pool())
        .await?
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: reports,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Get report by ID (moderators only)
pub async fn get_report(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Report>> {
    require_global_moderator(&state, user.identity_id).await?;

    let report = sqlx::query_as!(
        Report,
        r#"
        SELECT id, reporter_id, target_type as "target_type: ReportTargetType",
               target_id, reason as "reason: ReportReason", description,
               status as "status: ReportStatus", reviewed_by, review_notes, created_at, reviewed_at
        FROM reports
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Report not found".to_string()))?;

    Ok(Json(report))
}

/// Update report status (moderators only)
pub async fn update_report(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateReportRequest>,
) -> ApiResult<Json<Report>> {
    require_global_moderator(&state, user.identity_id).await?;

    let report = sqlx::query_as!(
        Report,
        r#"
        UPDATE reports
        SET status = $2, reviewed_by = $3, review_notes = $4, reviewed_at = NOW()
        WHERE id = $1
        RETURNING id, reporter_id, target_type as "target_type: ReportTargetType",
                  target_id, reason as "reason: ReportReason", description,
                  status as "status: ReportStatus", reviewed_by, review_notes, created_at, reviewed_at
        "#,
        id,
        request.status.to_string(),
        user.identity_id,
        request.notes
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(report))
}

/// Remove a post (moderator action)
pub async fn remove_post(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(request): Json<RemoveContentRequest>,
) -> ApiResult<StatusCode> {
    // Look up the post's space and verify moderator privileges
    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!(
        "UPDATE posts SET is_removed = true, removed_reason = $2 WHERE id = $1",
        id,
        request.reason
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

/// Remove a comment (moderator action)
pub async fn remove_comment(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(request): Json<RemoveContentRequest>,
) -> ApiResult<StatusCode> {
    // Look up the comment's post -> space to verify moderator privileges
    let comment = sqlx::query!("SELECT post_id FROM comments WHERE id = $1", id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Comment not found".to_string()))?;

    let post = sqlx::query!("SELECT space_id FROM posts WHERE id = $1", comment.post_id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| ApiError::NotFound("Post not found".to_string()))?;

    if !check_moderator(&state, user.identity_id, post.space_id).await? {
        return Err(ApiError::Forbidden);
    }

    sqlx::query!(
        "UPDATE comments SET is_removed = true, removed_reason = $2 WHERE id = $1",
        id,
        request.reason
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

/// Suspend an identity (global moderator only)
pub async fn suspend_identity(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(request): Json<SuspendRequest>,
) -> ApiResult<StatusCode> {
    require_global_moderator(&state, user.identity_id).await?;

    sqlx::query!(
        "UPDATE identities SET is_suspended = true, suspended_reason = $2, suspended_until = $3 WHERE id = $1",
        id,
        request.reason,
        request.until
    )
    .execute(state.db.pool())
    .await?;

    // Revoke all refresh tokens
    sqlx::query!("UPDATE refresh_tokens SET revoked = true WHERE identity_id = $1", id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::OK)
}

/// Unsuspend an identity (global moderator only)
pub async fn unsuspend_identity(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    require_global_moderator(&state, user.identity_id).await?;

    sqlx::query!(
        "UPDATE identities SET is_suspended = false, suspended_reason = NULL, suspended_until = NULL WHERE id = $1",
        id
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

// Request types

#[derive(Debug, serde::Deserialize)]
pub struct ReportListParams {
    pub status: Option<ReportStatus>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateReportRequest {
    pub status: ReportStatus,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RemoveContentRequest {
    pub reason: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SuspendRequest {
    pub reason: String,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Display for ReportTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ReportTargetType::Post => "post",
            ReportTargetType::Comment => "comment",
            ReportTargetType::Message => "message",
            ReportTargetType::Identity => "identity",
            ReportTargetType::Space => "space",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for ReportReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ReportReason::Spam => "spam",
            ReportReason::Harassment => "harassment",
            ReportReason::HateSpeech => "hate_speech",
            ReportReason::Violence => "violence",
            ReportReason::Misinformation => "misinformation",
            ReportReason::IllegalContent => "illegal_content",
            ReportReason::PrivacyViolation => "privacy_violation",
            ReportReason::Impersonation => "impersonation",
            ReportReason::Other => "other",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ReportStatus::Pending => "pending",
            ReportStatus::Reviewed => "reviewed",
            ReportStatus::Actioned => "actioned",
            ReportStatus::Dismissed => "dismissed",
        };
        write!(f, "{}", s)
    }
}
