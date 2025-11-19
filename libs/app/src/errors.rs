use axum::extract::rejection::JsonRejection;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
// use sqlx::Error as SqlxError;
use sea_orm::SqlxError;
use sea_orm_migration::DbErr;
use thiserror::Error;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Error as UuidError;
use validator::ValidationErrors;

pub mod messages {
    pub const VALIDATION_FAILED: &str = "Validation failed for the provided input.";
    pub const INVALID_UUID: &str = "Invalid id";
    pub const NOT_FOUND_RESOURCE: &str = "Requested resource was not found.";
    pub const INTERNAL_ERROR: &str = "An unexpected error occurred.";
    pub const GATEWAY_TIMEOUT: &str = "Gateway timeout occurred.";
    pub const INVALID_JSON: &str = "Invalid JSON format.";
    pub const DB_CONFIG_ERROR: &str = "Database configuration error.";
    pub const DB_ERROR: &str = "A database error occurred.";
    pub const DB_IO_ERROR: &str = "Database I/O error.";
    pub const DB_TLS_ERROR: &str = "Database TLS error.";
    pub const DB_PROTOCOL_ERROR: &str = "Database protocol error.";
    pub const DB_TYPE_NOT_FOUND: &str = "Database type not found.";
    pub const DB_DECODE_ERROR: &str = "Failed to decode database response.";
    pub const DB_ENCODE_ERROR: &str = "Failed to encode database request.";
    pub const DB_DRIVER_ERROR: &str = "A database driver error occurred.";
    pub const DB_POOL_TIMEOUT: &str = "Database connection pool timed out.";
    pub const DB_POOL_CLOSED: &str = "Database connection pool closed.";
    pub const DB_WORKER_CRASHED: &str = "Database connection pool worker crashed.";
    pub const DB_MIGRATION_ERROR: &str = "Database migration error.";
    pub const DB_INTERNAL_ERROR: &str = "Internal database error.";
    pub const AUTH_INVALID_CREDENTIALS: &str = "Invalid credentials.";
    pub const AUTH_TOKEN_INVALID: &str = "Invalid or expired token.";
    pub const AUTH_TOKEN_CREATION_FAILED: &str = "Failed to create authentication token.";
    pub const AUTH_UNAUTHORIZED: &str = "Authentication required.";

    pub const CODE_INFLUX_BUILD: i32 = 1;
    pub const CODE_INFLUX_REQUEST: i32 = 2;
    pub const CODE_MONGO: i32 = 3;
    pub const CODE_SQLX_NOT_FOUND: i32 = 4;
    pub const CODE_MIGRATION: i32 = 5;
    pub const CODE_IO: i32 = 6;
    pub const CODE_JSON_EXTRACTION: i32 = 7;
    pub const CODE_VALIDATION: i32 = 8;
    pub const CODE_UUID: i32 = 9;
    pub const CODE_NOT_FOUND: i32 = 10;
    pub const CODE_DATABASE_ERROR: i32 = 11;
    pub const CODE_INTERNAL: i32 = 12;
    pub const CODE_SQLX_CONFIG: i32 = 13;
    pub const CODE_SQLX_DATABASE: i32 = 14;
    pub const CODE_SQLX_IO: i32 = 15;
    pub const CODE_SQLX_TLS: i32 = 16;
    pub const CODE_SQLX_PROTOCOL: i32 = 17;
    pub const CODE_SQLX_TYPE_NOT_FOUND: i32 = 19;
    pub const CODE_SQLX_COLUMN_INDEX: i32 = 20;
    pub const CODE_SQLX_COLUMN_NOT_FOUND: i32 = 21;
    pub const CODE_SQLX_DECODE: i32 = 22;
    pub const CODE_SQLX_ENCODE: i32 = 23;
    pub const CODE_SQLX_DRIVER: i32 = 24;
    pub const CODE_SQLX_POOL_TIMEOUT: i32 = 25;
    pub const CODE_SQLX_POOL_CLOSED: i32 = 26;
    pub const CODE_SQLX_WORKER_CRASHED: i32 = 27;
    pub const CODE_SQLX_MIGRATE: i32 = 28;
    pub const CODE_SQLX_UNHANDLED: i32 = 29;
    pub const CODE_EYRE: i32 = 99;
    pub const CODE_SERDE_JSON: i32 = 211;
    pub const CODE_GATEWAY_TIMEOUT: i32 = 921;
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("SerdeJson parsing error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("InfluxDB connection error: {0}")]
    Influx(String),

    #[error("InfluxDB request error: {0}")]
    InfluxRequest(String),

    #[error("MongoDB connection error: {0}")]
    Mongo(String),

    #[error("Postgres connection error: {0}")]
    Postgres(#[from] SqlxError),

    #[error("Migration error: {0}")]
    Migration(#[from] DbErr),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON extraction error: {0}")]
    JsonExtractorRejection(#[from] JsonRejection),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),

    #[error("UUID error: {0}")]
    UuidError(#[from] UuidError),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Gateway Timeout")]
    GatewayTimeout,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error")]
    InternalError,

    #[error("Internal server error")]
    EyreError(#[from] eyre::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details, code) = match self {
            AppError::SerdeJson(e) => {
                error!(
                    error_code = messages::CODE_SERDE_JSON,
                    "Serde json parsing error: {:?}", e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    messages::INVALID_JSON.to_string(),
                    None,
                    messages::CODE_SERDE_JSON,
                )
            }
            AppError::Influx(e) => {
                error!(
                    error_code = messages::CODE_INFLUX_BUILD,
                    "InfluxDB connection error: {:?}", e
                );
                (
                    StatusCode::BAD_GATEWAY,
                    messages::DB_ERROR.to_string(),
                    None,
                    messages::CODE_INFLUX_BUILD,
                )
            }
            AppError::InfluxRequest(e) => {
                error!(
                    error_code = messages::CODE_INFLUX_REQUEST,
                    "InfluxDB request error: {:?}", e
                );
                (
                    StatusCode::BAD_GATEWAY,
                    messages::DB_ERROR.to_string(),
                    None,
                    messages::CODE_INFLUX_REQUEST,
                )
            }
            AppError::Mongo(e) => {
                error!(
                    error_code = messages::CODE_MONGO,
                    "MongoDB connection error: {:?}", e
                );
                (
                    StatusCode::BAD_GATEWAY,
                    messages::DB_ERROR.to_string(),
                    None,
                    messages::CODE_MONGO,
                )
            }
            AppError::GatewayTimeout => {
                error!(
                    error_code = messages::CODE_GATEWAY_TIMEOUT,
                    "Gateway Timeout"
                );
                (
                    StatusCode::GATEWAY_TIMEOUT,
                    messages::GATEWAY_TIMEOUT.to_string(),
                    None,
                    messages::CODE_GATEWAY_TIMEOUT,
                )
            }
            AppError::Postgres(e) => map_sqlx_error(&e),
            AppError::Migration(e) => {
                error!(
                    error_code = messages::CODE_MIGRATION,
                    "Postgres migration error: {:?}", e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    messages::DB_MIGRATION_ERROR.to_string(),
                    None,
                    messages::CODE_MIGRATION,
                )
            }
            AppError::Io(e) => {
                error!(error_code = messages::CODE_IO, "I/O error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    messages::INTERNAL_ERROR.to_string(),
                    None,
                    messages::CODE_IO,
                )
            }
            AppError::JsonExtractorRejection(e) => {
                warn!(
                    error_code = messages::CODE_JSON_EXTRACTION,
                    "JSON extraction error: {:?}", e
                );
                (
                    e.status(),
                    e.body_text(),
                    None,
                    messages::CODE_JSON_EXTRACTION,
                )
            }
            AppError::ValidationError(e) => {
                info!(
                    error_code = messages::CODE_VALIDATION,
                    "Validation error: {:?}", e
                );
                (
                    StatusCode::BAD_REQUEST,
                    messages::VALIDATION_FAILED.to_string(),
                    Some(serde_json::to_value(e).unwrap_or(serde_json::json!(null))),
                    messages::CODE_VALIDATION,
                )
            }
            AppError::UuidError(e) => {
                warn!(error_code = messages::CODE_UUID, "UUID error: {:?}", e);
                (
                    StatusCode::BAD_REQUEST,
                    messages::INVALID_UUID.to_string(),
                    None,
                    messages::CODE_UUID,
                )
            }
            AppError::NotFound(msg) => {
                info!(error_code = messages::CODE_NOT_FOUND, "Not Found: {}", msg);
                (StatusCode::NOT_FOUND, msg, None, messages::CODE_NOT_FOUND)
            }
            AppError::DatabaseError(msg) => {
                error!(
                    error_code = messages::CODE_DATABASE_ERROR,
                    "Database error: {}", msg
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg,
                    None,
                    messages::CODE_DATABASE_ERROR,
                )
            }
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::INTERNAL_ERROR.to_string(),
                None,
                messages::CODE_INTERNAL,
            ),
            AppError::EyreError(e) => {
                error!(error_code = messages::CODE_EYRE, "Eyre error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    messages::INTERNAL_ERROR.to_string(),
                    None,
                    messages::CODE_EYRE,
                )
            }
        };

        let api_error = ApiErrorMessage::new(
            status,
            status.canonical_reason().unwrap_or("Error").to_string(),
            error_message,
            details,
            Some(code),
        );
        (status, axum::Json(api_error)).into_response()
    }
}

fn map_sqlx_error(error: &SqlxError) -> (StatusCode, String, Option<serde_json::Value>, i32) {
    match error {
        SqlxError::RowNotFound => {
            info!(
                error_code = messages::CODE_SQLX_NOT_FOUND,
                "Postgres row not found: {:?}", error
            );
            (
                StatusCode::NOT_FOUND,
                messages::NOT_FOUND_RESOURCE.to_string(),
                None,
                messages::CODE_SQLX_NOT_FOUND,
            )
        }
        SqlxError::Configuration(e) => {
            error!(
                error_code = messages::CODE_SQLX_CONFIG,
                "Postgres configuration error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_CONFIG_ERROR.to_string(),
                None,
                messages::CODE_SQLX_CONFIG,
            )
        }
        SqlxError::Database(e) => {
            error!(
                error_code = messages::CODE_SQLX_DATABASE,
                "Postgres database error: {:?}", e
            );
            (
                StatusCode::BAD_GATEWAY,
                messages::DB_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DATABASE,
            )
        }
        SqlxError::Io(e) => {
            error!(
                error_code = messages::CODE_SQLX_IO,
                "Postgres I/O error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_IO_ERROR.to_string(),
                None,
                messages::CODE_SQLX_IO,
            )
        }
        SqlxError::Tls(e) => {
            error!(
                error_code = messages::CODE_SQLX_TLS,
                "Postgres TLS error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_TLS_ERROR.to_string(),
                None,
                messages::CODE_SQLX_TLS,
            )
        }
        SqlxError::Protocol(e) => {
            error!(
                error_code = messages::CODE_SQLX_PROTOCOL,
                "Postgres protocol error: {:?}", e
            );
            (
                StatusCode::BAD_GATEWAY,
                messages::DB_PROTOCOL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_PROTOCOL,
            )
        }
        SqlxError::TypeNotFound { type_name } => {
            error!(
                error_code = messages::CODE_SQLX_TYPE_NOT_FOUND,
                "Postgres type not found: type_name={}", type_name
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_TYPE_NOT_FOUND.to_string(),
                None,
                messages::CODE_SQLX_TYPE_NOT_FOUND,
            )
        }
        SqlxError::ColumnIndexOutOfBounds { index, len } => {
            error!(
                error_code = messages::CODE_SQLX_COLUMN_INDEX,
                "Postgres column index out of bounds: index={}, len={}", index, len
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_INTERNAL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_COLUMN_INDEX,
            )
        }
        SqlxError::ColumnNotFound(column) => {
            error!(
                error_code = messages::CODE_SQLX_COLUMN_NOT_FOUND,
                "Postgres column not found: {}", column
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_INTERNAL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_COLUMN_NOT_FOUND,
            )
        }
        SqlxError::Decode(e) => {
            warn!(
                error_code = messages::CODE_SQLX_DECODE,
                "Postgres decode error: {:?}", e
            );
            (
                StatusCode::BAD_REQUEST,
                messages::DB_DECODE_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DECODE,
            )
        }
        SqlxError::Encode(e) => {
            warn!(
                error_code = messages::CODE_SQLX_ENCODE,
                "Postgres encode error: {:?}", e
            );
            (
                StatusCode::BAD_REQUEST,
                messages::DB_ENCODE_ERROR.to_string(),
                None,
                messages::CODE_SQLX_ENCODE,
            )
        }
        SqlxError::AnyDriverError(e) => {
            error!(
                error_code = messages::CODE_SQLX_DRIVER,
                "Postgres driver error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_DRIVER_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DRIVER,
            )
        }
        SqlxError::PoolTimedOut => {
            warn!(
                error_code = messages::CODE_SQLX_POOL_TIMEOUT,
                "Postgres connection pool timed out"
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                messages::DB_POOL_TIMEOUT.to_string(),
                None,
                messages::CODE_SQLX_POOL_TIMEOUT,
            )
        }
        SqlxError::PoolClosed => {
            error!(
                error_code = messages::CODE_SQLX_POOL_CLOSED,
                "Postgres connection pool has been closed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_POOL_CLOSED.to_string(),
                None,
                messages::CODE_SQLX_POOL_CLOSED,
            )
        }
        SqlxError::WorkerCrashed => {
            error!(
                error_code = messages::CODE_SQLX_WORKER_CRASHED,
                "Postgres connection pool worker crashed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_WORKER_CRASHED.to_string(),
                None,
                messages::CODE_SQLX_WORKER_CRASHED,
            )
        }
        SqlxError::Migrate(e) => {
            error!(
                error_code = messages::CODE_SQLX_MIGRATE,
                "Postgres migration error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_MIGRATION_ERROR.to_string(),
                None,
                messages::CODE_SQLX_MIGRATE,
            )
        }
        _ => {
            error!(
                error_code = messages::CODE_SQLX_UNHANDLED,
                "Unhandled Postgres error: {:?}", error
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                messages::DB_ERROR.to_string(),
                None,
                messages::CODE_SQLX_UNHANDLED,
            )
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorMessage {
    /// HTTP status code
    pub status: u16,
    /// Short error type or title
    pub error: String,
    /// Detailed human-readable message
    pub message: String,
    /// Optional field for additional info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Custom error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}

impl ApiErrorMessage {
    pub fn new(
        status: StatusCode,
        error: impl Into<String>,
        message: impl Into<String>,
        details: Option<impl serde::Serialize>,
        code: Option<i32>,
    ) -> Self {
        Self {
            status: status.as_u16(),
            error: error.into(),
            message: message.into(),
            details: details.map(|d| serde_json::to_value(d).unwrap_or(serde_json::json!(null))),
            code,
        }
    }
}

pub trait SecureError {
    fn to_secure_response(&self) -> (StatusCode, String);
}

// impl SecureError for SqlxError {
//     fn to_secure_response(&self) -> (StatusCode, String) {
//         let (status, message, _, _) = map_sqlx_error(self);
//         (status, message)
//     }
// }
