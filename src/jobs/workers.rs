//! Background job workers

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

use crate::AppState;

/// Start all background workers
pub async fn start_background_workers(state: Arc<AppState>) {
    tokio::spawn(cleanup_worker(state.clone()));
    tokio::spawn(score_update_worker(state.clone()));

    info!("Background workers started");
}

/// Cleanup worker - removes expired tokens and old data
async fn cleanup_worker(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(3600)); // Every hour

    loop {
        ticker.tick().await;

        // Clean up expired refresh tokens
        match sqlx::query!(
            "DELETE FROM refresh_tokens WHERE expires_at < NOW() OR revoked = true"
        )
        .execute(state.db.pool())
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    info!(count = result.rows_affected(), "Cleaned up expired refresh tokens");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to clean up refresh tokens");
            }
        }

        // Clean up old notifications (older than 30 days)
        match sqlx::query!(
            "DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '30 days' AND is_read = true"
        )
        .execute(state.db.pool())
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    info!(count = result.rows_affected(), "Cleaned up old notifications");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to clean up notifications");
            }
        }

        // Clean up temp files
        if let Err(e) = state.storage.cleanup_temp_files(Duration::from_secs(86400)).await {
            error!(error = %e, "Failed to clean up temp files");
        }

        debug!("Cleanup worker completed cycle");
    }
}

/// Score update worker - recalculates hot scores periodically
async fn score_update_worker(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(300)); // Every 5 minutes

    loop {
        ticker.tick().await;

        // Update post scores based on age and votes
        // Hot score decays over time
        match sqlx::query!(
            r#"
            UPDATE posts
            SET score = (
                CASE
                    WHEN upvotes - downvotes > 0 THEN
                        LOG(GREATEST(ABS(upvotes - downvotes), 1)) +
                        EXTRACT(EPOCH FROM created_at) / 45000.0
                    WHEN upvotes - downvotes < 0 THEN
                        -LOG(GREATEST(ABS(upvotes - downvotes), 1)) +
                        EXTRACT(EPOCH FROM created_at) / 45000.0
                    ELSE
                        EXTRACT(EPOCH FROM created_at) / 45000.0
                END
            )::INTEGER
            WHERE created_at > NOW() - INTERVAL '7 days'
            "#
        )
        .execute(state.db.pool())
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    debug!(count = result.rows_affected(), "Updated post scores");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to update post scores");
            }
        }

        debug!("Score update worker completed cycle");
    }
}

/// Job for sending notification
pub async fn send_notification_job(
    state: &Arc<AppState>,
    recipient_id: uuid::Uuid,
    notification_type: crate::domain::entities::NotificationType,
    payload: serde_json::Value,
) -> Result<(), crate::errors::ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO notifications (id, recipient_id, notification_type, payload, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        uuid::Uuid::new_v4(),
        recipient_id,
        notification_type.to_string(),
        payload
    )
    .execute(state.db.pool())
    .await?;

    // Broadcast to WebSocket if connected
    crate::websocket::broadcast_notification(
        state,
        crate::websocket::NotificationMessage {
            recipient_id,
            notification_type: notification_type.to_string(),
            payload,
            created_at: chrono::Utc::now(),
        },
    ).await;

    Ok(())
}

impl ToString for crate::domain::entities::NotificationType {
    fn to_string(&self) -> String {
        match self {
            crate::domain::entities::NotificationType::PostReply => "post_reply",
            crate::domain::entities::NotificationType::CommentReply => "comment_reply",
            crate::domain::entities::NotificationType::Mention => "mention",
            crate::domain::entities::NotificationType::NewMessage => "new_message",
            crate::domain::entities::NotificationType::SpaceInvite => "space_invite",
            crate::domain::entities::NotificationType::ModeratorAction => "moderator_action",
            crate::domain::entities::NotificationType::SystemAlert => "system_alert",
        }.to_string()
    }
}
