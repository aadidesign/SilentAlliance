//! Media upload handlers with metadata stripping

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::Media;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// Upload media with metadata stripping
pub async fn upload(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<Media>)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::InvalidInput(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();

        let data = field.bytes().await.map_err(|e| {
            ApiError::InvalidInput(format!("Failed to read file data: {}", e))
        })?;

        // Store with metadata stripping
        let stored = state.storage.store_image(&data, filename.as_deref(), &content_type).await?;

        // Save to database
        let media = sqlx::query_as!(
            Media,
            r#"
            INSERT INTO media (id, uploader_id, file_hash, mime_type, file_size, storage_path, thumbnail_path, is_processed, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW())
            RETURNING id, uploader_id, file_hash, mime_type, file_size, storage_path, thumbnail_path, is_processed, created_at
            "#,
            stored.id,
            user.identity_id,
            stored.content_hash,
            stored.mime_type,
            stored.size as i32,
            stored.path,
            stored.thumbnail_path
        )
        .fetch_one(state.db.pool())
        .await?;

        return Ok((StatusCode::CREATED, Json(media)));
    }

    Err(ApiError::InvalidInput("No file provided".to_string()))
}

/// Get media metadata by ID
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Media>> {
    let media = sqlx::query_as!(
        Media,
        "SELECT id, uploader_id, file_hash, mime_type, file_size, storage_path, thumbnail_path, is_processed, created_at FROM media WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    Ok(Json(media))
}

/// Download media file
pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let media = sqlx::query!(
        "SELECT storage_path, mime_type FROM media WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    let data = state.storage.get_file(&media.storage_path).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(data))
        .unwrap())
}

/// Get thumbnail
pub async fn get_thumbnail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let media = sqlx::query!(
        "SELECT thumbnail_path, mime_type FROM media WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    let thumbnail_path = media.thumbnail_path
        .ok_or_else(|| ApiError::NotFound("Thumbnail not available".to_string()))?;

    let data = state.storage.get_file(&thumbnail_path).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(data))
        .unwrap())
}
