//! Error types for the notifications domain.

use thiserror::Error;
use uuid::Uuid;

/// Result type for notification operations.
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Errors that can occur in the notifications domain.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Email address is suppressed (bounced, complained, or unsubscribed).
    #[error("Email address is suppressed: {0}")]
    EmailSuppressed(String),

    /// User has disabled this notification type.
    #[error("User has disabled {0} notifications")]
    NotificationDisabled(String),

    /// Verification token not found.
    #[error("Verification token not found")]
    TokenNotFound,

    /// Verification token has expired.
    #[error("Verification token has expired")]
    TokenExpired,

    /// Verification token has already been used.
    #[error("Verification token has already been used")]
    TokenAlreadyUsed,

    /// Rate limit exceeded.
    #[error("Rate limit exceeded. Please try again later")]
    RateLimitExceeded,

    /// Email provider error.
    #[error("Email provider error: {0}")]
    ProviderError(String),

    /// Template rendering error.
    #[error("Template rendering error: {0}")]
    TemplateError(String),

    /// Redis queue error.
    #[error("Queue error: {0}")]
    QueueError(String),

    /// Database error.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// User not found.
    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    /// Invalid email address.
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<redis::RedisError> for NotificationError {
    fn from(err: redis::RedisError) -> Self {
        NotificationError::QueueError(err.to_string())
    }
}

impl From<sea_orm::DbErr> for NotificationError {
    fn from(err: sea_orm::DbErr) -> Self {
        NotificationError::DatabaseError(err.to_string())
    }
}

impl From<handlebars::RenderError> for NotificationError {
    fn from(err: handlebars::RenderError) -> Self {
        NotificationError::TemplateError(err.to_string())
    }
}

impl From<reqwest::Error> for NotificationError {
    fn from(err: reqwest::Error) -> Self {
        NotificationError::ProviderError(err.to_string())
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(err: serde_json::Error) -> Self {
        NotificationError::Internal(format!("JSON serialization error: {}", err))
    }
}

impl From<std::io::Error> for NotificationError {
    fn from(err: std::io::Error) -> Self {
        NotificationError::ProviderError(format!("IO error: {}", err))
    }
}
