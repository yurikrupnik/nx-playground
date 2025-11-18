//! Generic health check utilities for Axum applications.

use axum::{http::StatusCode, Json};
use futures::future::join_all;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// A boxed future for health checks with a string error
pub type HealthCheckFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

/// Runs multiple health checks concurrently and returns aggregated results.
///
/// # Arguments
/// * `checks` - Vector of (name, check_future) tuples
///
/// # Example
/// ```ignore
/// let checks = vec![
///     ("database", Box::pin(async {
///         check_database(db).await.map_err(|e| e.to_string())
///     })),
///     ("redis", Box::pin(async {
///         check_redis(redis).await.map_err(|e| e.to_string())
///     })),
/// ];
/// run_health_checks(checks).await
/// ```
pub async fn run_health_checks(
    checks: Vec<(&str, HealthCheckFuture<'_>)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Run all checks concurrently
    let names: Vec<_> = checks.iter().map(|(name, _)| *name).collect();
    let futures: Vec<_> = checks.into_iter().map(|(_, check)| check).collect();
    let results = join_all(futures).await;

    // Aggregate results
    let mut status_map = HashMap::new();
    let mut all_healthy = true;

    for (name, result) in names.into_iter().zip(results) {
        match result {
            Ok(_) => {
                status_map.insert(name, "connected");
            }
            Err(e) => {
                tracing::error!("Readiness check failed: {} error: {:?}", name, e);
                status_map.insert(name, "disconnected");
                all_healthy = false;
            }
        }
    }

    let mut response = json!({
        "status": if all_healthy { "ready" } else { "not ready" }
    });

    // Add each check result to response
    if let Value::Object(ref mut map) = response {
        for (name, status) in status_map {
            map.insert(name.to_string(), json!(status));
        }
    }

    if all_healthy {
        Ok((StatusCode::OK, Json(response)))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}
