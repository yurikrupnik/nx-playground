use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "healthy")
}
