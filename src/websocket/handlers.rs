//! WebSocket connection handlers
//!
//! Handles authenticated WebSocket connections with:
//! - Heartbeat/ping-pong keepalive
//! - Per-connection message rate limiting
//! - Maximum message size enforcement
//! - Graceful disconnection handling

use futures::{sink::SinkExt, stream::StreamExt};
use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::AppState;

/// Maximum allowed size for a single WebSocket text message (64 KB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Maximum messages a client can send per window (burst protection)
const MAX_MESSAGES_PER_WINDOW: u32 = 30;

/// Rate limit window duration
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);

/// Heartbeat interval
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Notification message for broadcasting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationMessage {
    pub recipient_id: Uuid,
    pub notification_type: String,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Simple sliding-window rate limiter for a single connection
struct ConnectionRateLimiter {
    window_start: Instant,
    count: u32,
}

impl ConnectionRateLimiter {
    fn new() -> Self {
        Self {
            window_start: Instant::now(),
            count: 0,
        }
    }

    /// Returns `true` if the message is allowed, `false` if rate limited.
    fn check(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.window_start) > RATE_LIMIT_WINDOW {
            // Reset the window
            self.window_start = now;
            self.count = 0;
        }
        self.count += 1;
        self.count <= MAX_MESSAGES_PER_WINDOW
    }
}

/// Handle an authenticated WebSocket connection
pub async fn handle_authenticated_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    identity_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rate_limiter = ConnectionRateLimiter::new();

    info!(identity_id = %identity_id, "WebSocket connection established");

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "identity_id": identity_id.to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if sender.send(Message::Text(welcome.to_string())).await.is_err() {
        return;
    }

    // Main loop - handle heartbeats and messages
    loop {
        tokio::select! {
            // Heartbeat every 30 seconds
            _ = tokio::time::sleep(HEARTBEAT_INTERVAL) => {
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }

            // Handle incoming messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Enforce message size limit
                        if text.len() > MAX_MESSAGE_SIZE {
                            warn!(
                                identity_id = %identity_id,
                                size = text.len(),
                                "WebSocket message exceeds size limit"
                            );
                            let err = serde_json::json!({
                                "type": "error",
                                "message": "Message too large",
                                "max_size": MAX_MESSAGE_SIZE
                            });
                            let _ = sender.send(Message::Text(err.to_string())).await;
                            continue;
                        }

                        // Enforce per-connection rate limit
                        if !rate_limiter.check() {
                            warn!(
                                identity_id = %identity_id,
                                "WebSocket rate limit exceeded"
                            );
                            let err = serde_json::json!({
                                "type": "error",
                                "message": "Rate limit exceeded, slow down"
                            });
                            let _ = sender.send(Message::Text(err.to_string())).await;
                            continue;
                        }

                        // Handle client messages (e.g., subscribe to channels)
                        if let Err(e) = handle_client_message(&text, identity_id, &state).await {
                            warn!(error = %e, "Failed to handle client message");
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        // Reject binary messages — we only support JSON text
                        if data.len() > MAX_MESSAGE_SIZE {
                            warn!(identity_id = %identity_id, "Oversized binary message rejected");
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!(identity_id = %identity_id, "Received pong");
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!(identity_id = %identity_id, "WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "WebSocket error");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_client_message(
    text: &str,
    identity_id: Uuid,
    _state: &Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message: serde_json::Value = serde_json::from_str(text)?;

    match message.get("type").and_then(|t| t.as_str()) {
        Some("subscribe") => {
            // Handle channel subscription
            if let Some(channel) = message.get("channel").and_then(|c| c.as_str()) {
                debug!(identity_id = %identity_id, channel = %channel, "Subscribe request");
            }
        }
        Some("unsubscribe") => {
            // Handle channel unsubscription
            if let Some(channel) = message.get("channel").and_then(|c| c.as_str()) {
                debug!(identity_id = %identity_id, channel = %channel, "Unsubscribe request");
            }
        }
        Some("ping") => {
            debug!(identity_id = %identity_id, "Client ping");
        }
        _ => {
            warn!(identity_id = %identity_id, "Unknown message type");
        }
    }

    Ok(())
}

/// Broadcast a notification to connected clients
pub async fn broadcast_notification(
    _state: &Arc<AppState>,
    notification: NotificationMessage,
) {
    // In production, this would use Redis pub/sub or a dedicated message broker
    // to distribute notifications to all server instances
    debug!(
        recipient = %notification.recipient_id,
        notification_type = %notification.notification_type,
        "Broadcasting notification"
    );
}
