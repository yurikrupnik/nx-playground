//! HTTP handlers for health checks and admin endpoints

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use email::provider::EmailProvider;
use email::stream;
use serde::Serialize;

use crate::state::AppState;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub redis: String,
    pub smtp: String,
}

/// Health check endpoint
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let mut redis_conn = state.redis.clone();

    // Check Redis
    let redis_status = match redis::cmd("PING")
        .query_async::<String>(&mut redis_conn)
        .await
    {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };

    // Check SMTP
    let smtp_status = match state.provider.health_check().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    };

    let status = if redis_status == "ok" && smtp_status == "ok" {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthResponse {
        status: status.to_string(),
        redis: redis_status,
        smtp: smtp_status,
    };

    if status == "healthy" {
        (StatusCode::OK, Json(response))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// Stream info response
#[derive(Serialize)]
pub struct StreamInfoResponse {
    pub length: usize,
    pub first_entry_id: Option<String>,
    pub last_entry_id: Option<String>,
}

/// Get stream info endpoint
pub async fn stream_info(State(state): State<AppState>) -> impl IntoResponse {
    let mut redis = state.redis.clone();

    match stream::get_stream_info(&mut redis).await {
        Ok(info) => (
            StatusCode::OK,
            Json(StreamInfoResponse {
                length: info.length,
                first_entry_id: info.first_entry_id,
                last_entry_id: info.last_entry_id,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(StreamInfoResponse {
                length: 0,
                first_entry_id: None,
                last_entry_id: Some(format!("error: {}", e)),
            }),
        ),
    }
}
