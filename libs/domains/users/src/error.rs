use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(Uuid),

    #[error("User with email '{0}' already exists")]
    DuplicateEmail(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Password hashing error: {0}")]
    PasswordHash(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type UserResult<T> = Result<T, UserError>;

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            UserError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("User {} not found", id),
            ),
            UserError::DuplicateEmail(email) => (
                StatusCode::CONFLICT,
                "duplicate",
                format!("User with email '{}' already exists", email),
            ),
            UserError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "Invalid email or password".to_string(),
            ),
            UserError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg.clone())
            }
            UserError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Unauthorized".to_string(),
            ),
            UserError::PasswordHash(msg) => {
                tracing::error!("Password hash error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
            UserError::OAuth(msg) => {
                tracing::error!("OAuth error: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    "oauth_error",
                    format!("OAuth authentication failed: {}", msg),
                )
            }
            UserError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": {
                    "type": error_type,
                    "message": message
                }
            })),
        )
            .into_response()
    }
}
