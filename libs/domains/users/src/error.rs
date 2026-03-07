use axum_helpers::{AppError, impl_into_response_via_app_error};
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

impl From<UserError> for AppError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound(id) => AppError::NotFound(format!("User {} not found", id)),
            UserError::DuplicateEmail(email) => {
                AppError::Conflict(format!("User with email '{}' already exists", email))
            }
            UserError::InvalidCredentials => {
                AppError::Unauthorized("Invalid email or password".to_string())
            }
            UserError::Validation(msg) => AppError::BadRequest(msg),
            UserError::Unauthorized => AppError::Unauthorized("Unauthorized".to_string()),
            UserError::PasswordHash(msg) => {
                tracing::error!("Password hash error: {}", msg);
                AppError::InternalServerError("An internal error occurred".to_string())
            }
            UserError::OAuth(msg) => {
                tracing::error!("OAuth error: {}", msg);
                AppError::Unauthorized(format!("OAuth authentication failed: {}", msg))
            }
            UserError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                AppError::InternalServerError("An internal error occurred".to_string())
            }
        }
    }
}

impl_into_response_via_app_error!(UserError);
