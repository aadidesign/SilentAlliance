//! Notification handlers with WebSocket support

use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// List notifications
pub async fn list(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<Notification>>> {
    let notifications = sqlx::query_as!(
        Notification,
        r#"
        SELECT id, recipient_id, notification_type as "notification_type: NotificationType",
               payload, is_read, created_at
        FROM notifications
        WHERE recipient_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.identity_id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notifications WHERE recipient_id = $1",
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: notifications,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Get unread count
pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> ApiResult<Json<UnreadCountResponse>> {
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM notifications WHERE recipient_id = $1 AND is_read = false",
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(UnreadCountResponse { count }))
}

/// Mark notification as read
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query!(
        "UPDATE notifications SET is_read = true WHERE id = $1 AND recipient_id = $2",
        id,
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Notification not found".to_string()));
    }

    Ok(StatusCode::OK)
}

/// Mark all notifications as read
pub async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    sqlx::query!(
        "UPDATE notifications SET is_read = true WHERE recipient_id = $1 AND is_read = false",
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

/// WebSocket handler for real-time notifications
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Authentication via first message
    let auth_msg = match receiver.next().await {
        Some(Ok(Message::Text(token))) => token,
        _ => return,
    };

    // Validate token and get identity
    let jwt_service = match crate::domain::services::auth::JwtService::new(&state.settings.jwt) {
        Ok(s) => s,
        Err(_) => return,
    };

    let claims = match jwt_service.validate_access_token(&auth_msg) {
        Ok(c) => c,
        Err(_) => {
            let _ = sender.send(Message::Text(r#"{"error":"invalid_token"}"#.to_string())).await;
            return;
        }
    };

    let identity_id: Uuid = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => return,
    };

    // Send confirmation
    let _ = sender.send(Message::Text(r#"{"status":"connected"}"#.to_string())).await;

    // Subscribe to notifications via Redis pub/sub (simplified)
    let channel = format!("notifications:{}", identity_id);

    // In production, you would use Redis pub/sub here
    // For now, just keep connection alive with heartbeats
    loop {
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Pong(_))) => continue,
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => continue,
                }
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}
