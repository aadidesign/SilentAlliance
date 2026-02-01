//! Health check endpoints
//!
//! Provides endpoints for monitoring and health checks.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<ServiceStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<ServiceStatus>,
}

/// Service status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

/// Basic health check - always returns OK if the server is running
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: None,
        redis: None,
    })
}

/// Readiness check - checks if all dependencies are ready
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    let mut is_ready = true;

    // Check database
    let db_start = std::time::Instant::now();
    let db_status = match state.db.health_check().await {
        Ok(_) => ServiceStatus {
            status: "healthy".to_string(),
            latency_ms: Some(db_start.elapsed().as_millis() as u64),
        },
        Err(_) => {
            is_ready = false;
            ServiceStatus {
                status: "unhealthy".to_string(),
                latency_ms: None,
            }
        }
    };

    // Check Redis
    let redis_start = std::time::Instant::now();
    let redis_status = match state.redis.health_check().await {
        Ok(_) => ServiceStatus {
            status: "healthy".to_string(),
            latency_ms: Some(redis_start.elapsed().as_millis() as u64),
        },
        Err(_) => {
            is_ready = false;
            ServiceStatus {
                status: "unhealthy".to_string(),
                latency_ms: None,
            }
        }
    };

    let response = HealthResponse {
        status: if is_ready { "ready" } else { "not_ready" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: Some(db_status),
        redis: Some(redis_status),
    };

    if is_ready {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

/// Liveness check - checks if the server is alive
pub async fn liveness_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: None,
        redis: None,
    })
}
