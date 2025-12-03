pub mod handlers;
pub mod messages;
pub mod responses;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::SqlxError;
use sea_orm_migration::DbErr;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Error as UuidError;
use validator::ValidationErrors;

/// Standard error response structure.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Custom error code for debugging and monitoring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}

/// Application error type that can be converted to HTTP responses.
///
/// This enum integrates with common error types from dependencies
/// and provides structured error responses with error codes for observability.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("JSON parsing error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

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

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable Entity: {0}")]
    UnprocessableEntity(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details, code) = match self {
            AppError::SerdeJson(e) => {
                tracing::error!(
                    error_code = messages::CODE_SERDE_JSON,
                    "JSON parsing error: {:?}", e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    messages::INVALID_JSON.to_string(),
                    None,
                    messages::CODE_SERDE_JSON,
                )
            }
            AppError::Database(e) => map_sqlx_error(&e),
            AppError::Migration(e) => {
                tracing::error!(
                    error_code = messages::CODE_MIGRATION,
                    "Database migration error: {:?}", e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    messages::DB_MIGRATION_ERROR.to_string(),
                    None,
                    messages::CODE_MIGRATION,
                )
            }
            AppError::Io(e) => {
                tracing::error!(
                    error_code = messages::CODE_IO,
                    "I/O error: {:?}", e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    messages::INTERNAL_ERROR.to_string(),
                    None,
                    messages::CODE_IO,
                )
            }
            AppError::JsonExtractorRejection(e) => {
                tracing::warn!(
                    error_code = messages::CODE_JSON_EXTRACTION,
                    "JSON extraction error: {:?}", e
                );
                (
                    e.status(),
                    "BadRequest",
                    e.body_text(),
                    None,
                    messages::CODE_JSON_EXTRACTION,
                )
            }
            AppError::ValidationError(e) => {
                tracing::info!(
                    error_code = messages::CODE_VALIDATION,
                    "Validation error: {:?}", e
                );
                (
                    StatusCode::BAD_REQUEST,
                    "BadRequest",
                    messages::VALIDATION_FAILED.to_string(),
                    Some(serde_json::to_value(&e).unwrap_or(serde_json::json!(null))),
                    messages::CODE_VALIDATION,
                )
            }
            AppError::UuidError(e) => {
                tracing::warn!(
                    error_code = messages::CODE_UUID,
                    "UUID error: {:?}", e
                );
                (
                    StatusCode::BAD_REQUEST,
                    "BadRequest",
                    messages::INVALID_UUID.to_string(),
                    None,
                    messages::CODE_UUID,
                )
            }
            AppError::BadRequest(msg) => {
                tracing::info!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, "BadRequest", msg, None, messages::CODE_INTERNAL)
            }
            AppError::Unauthorized(msg) => {
                tracing::info!("Unauthorized: {}", msg);
                (StatusCode::UNAUTHORIZED, "Unauthorized", msg, None, messages::CODE_INTERNAL)
            }
            AppError::Forbidden(msg) => {
                tracing::info!("Forbidden: {}", msg);
                (StatusCode::FORBIDDEN, "Forbidden", msg, None, messages::CODE_INTERNAL)
            }
            AppError::NotFound(msg) => {
                tracing::info!(
                    error_code = messages::CODE_NOT_FOUND,
                    "Not found: {}", msg
                );
                (StatusCode::NOT_FOUND, "NotFound", msg, None, messages::CODE_NOT_FOUND)
            }
            AppError::Conflict(msg) => {
                tracing::info!("Conflict: {}", msg);
                (StatusCode::CONFLICT, "Conflict", msg, None, messages::CODE_INTERNAL)
            }
            AppError::UnprocessableEntity(msg) => {
                tracing::info!("Unprocessable entity: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "UnprocessableEntity",
                    msg,
                    None,
                    messages::CODE_INTERNAL,
                )
            }
            AppError::InternalServerError(msg) => {
                tracing::error!(
                    error_code = messages::CODE_INTERNAL,
                    "Internal server error: {}", msg
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    msg,
                    None,
                    messages::CODE_INTERNAL,
                )
            }
            AppError::ServiceUnavailable(msg) => {
                tracing::warn!("Service unavailable: {}", msg);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "ServiceUnavailable",
                    msg,
                    None,
                    messages::CODE_INTERNAL,
                )
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
            details,
            code: Some(code),
        });

        (status, body).into_response()
    }
}

/// Maps SqlxError to appropriate HTTP response components.
///
/// This function provides detailed error handling for all SqlxError variants,
/// with appropriate status codes, messages, and error codes for observability.
fn map_sqlx_error(
    error: &SqlxError,
) -> (StatusCode, &'static str, String, Option<serde_json::Value>, i32) {
    match error {
        SqlxError::RowNotFound => {
            tracing::info!(
                error_code = messages::CODE_SQLX_NOT_FOUND,
                "Database row not found"
            );
            (
                StatusCode::NOT_FOUND,
                "NotFound",
                messages::NOT_FOUND_RESOURCE.to_string(),
                None,
                messages::CODE_SQLX_NOT_FOUND,
            )
        }
        SqlxError::Configuration(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_CONFIG,
                "Database configuration error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_CONFIG_ERROR.to_string(),
                None,
                messages::CODE_SQLX_CONFIG,
            )
        }
        SqlxError::Database(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_DATABASE,
                "Database error: {:?}", e
            );
            (
                StatusCode::BAD_GATEWAY,
                "BadGateway",
                messages::DB_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DATABASE,
            )
        }
        SqlxError::Io(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_IO,
                "Database I/O error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_IO_ERROR.to_string(),
                None,
                messages::CODE_SQLX_IO,
            )
        }
        SqlxError::Tls(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_TLS,
                "Database TLS error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_TLS_ERROR.to_string(),
                None,
                messages::CODE_SQLX_TLS,
            )
        }
        SqlxError::Protocol(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_PROTOCOL,
                "Database protocol error: {:?}", e
            );
            (
                StatusCode::BAD_GATEWAY,
                "BadGateway",
                messages::DB_PROTOCOL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_PROTOCOL,
            )
        }
        SqlxError::TypeNotFound { type_name } => {
            tracing::error!(
                error_code = messages::CODE_SQLX_TYPE_NOT_FOUND,
                "Database type not found: type_name={}", type_name
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_TYPE_NOT_FOUND.to_string(),
                None,
                messages::CODE_SQLX_TYPE_NOT_FOUND,
            )
        }
        SqlxError::ColumnIndexOutOfBounds { index, len } => {
            tracing::error!(
                error_code = messages::CODE_SQLX_COLUMN_INDEX,
                "Database column index out of bounds: index={}, len={}", index, len
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_INTERNAL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_COLUMN_INDEX,
            )
        }
        SqlxError::ColumnNotFound(column) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_COLUMN_NOT_FOUND,
                "Database column not found: {}", column
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_INTERNAL_ERROR.to_string(),
                None,
                messages::CODE_SQLX_COLUMN_NOT_FOUND,
            )
        }
        SqlxError::Decode(e) => {
            tracing::warn!(
                error_code = messages::CODE_SQLX_DECODE,
                "Database decode error: {:?}", e
            );
            (
                StatusCode::BAD_REQUEST,
                "BadRequest",
                messages::DB_DECODE_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DECODE,
            )
        }
        SqlxError::Encode(e) => {
            tracing::warn!(
                error_code = messages::CODE_SQLX_ENCODE,
                "Database encode error: {:?}", e
            );
            (
                StatusCode::BAD_REQUEST,
                "BadRequest",
                messages::DB_ENCODE_ERROR.to_string(),
                None,
                messages::CODE_SQLX_ENCODE,
            )
        }
        SqlxError::AnyDriverError(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_DRIVER,
                "Database driver error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_DRIVER_ERROR.to_string(),
                None,
                messages::CODE_SQLX_DRIVER,
            )
        }
        SqlxError::PoolTimedOut => {
            tracing::warn!(
                error_code = messages::CODE_SQLX_POOL_TIMEOUT,
                "Database connection pool timed out"
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "ServiceUnavailable",
                messages::DB_POOL_TIMEOUT.to_string(),
                None,
                messages::CODE_SQLX_POOL_TIMEOUT,
            )
        }
        SqlxError::PoolClosed => {
            tracing::error!(
                error_code = messages::CODE_SQLX_POOL_CLOSED,
                "Database connection pool has been closed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_POOL_CLOSED.to_string(),
                None,
                messages::CODE_SQLX_POOL_CLOSED,
            )
        }
        SqlxError::WorkerCrashed => {
            tracing::error!(
                error_code = messages::CODE_SQLX_WORKER_CRASHED,
                "Database connection pool worker crashed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_WORKER_CRASHED.to_string(),
                None,
                messages::CODE_SQLX_WORKER_CRASHED,
            )
        }
        SqlxError::Migrate(e) => {
            tracing::error!(
                error_code = messages::CODE_SQLX_MIGRATE,
                "Database migration error: {:?}", e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_MIGRATION_ERROR.to_string(),
                None,
                messages::CODE_SQLX_MIGRATE,
            )
        }
        _ => {
            tracing::error!(
                error_code = messages::CODE_SQLX_UNHANDLED,
                "Unhandled database error: {:?}", error
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError",
                messages::DB_ERROR.to_string(),
                None,
                messages::CODE_SQLX_UNHANDLED,
            )
        }
    }
}

/// Helper function to create error responses.
pub fn error_response(
    status: StatusCode,
    error: &str,
    message: String,
    code: Option<i32>,
) -> Response {
    let body = Json(ErrorResponse {
        error: error.to_string(),
        message,
        details: None,
        code,
    });

    (status, body).into_response()
}
