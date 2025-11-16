use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Database(DbErr),
    NotFound(String),
    Validation(String),
    Internal(String),
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::Database(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                match err {
                    DbErr::RecordNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Record not found".to_string())
                    }
                    DbErr::Query(e) if e.to_string().contains("duplicate key") => (
                        StatusCode::CONFLICT,
                        "A record with this username or email already exists".to_string(),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error occurred".to_string(),
                    ),
                }
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
