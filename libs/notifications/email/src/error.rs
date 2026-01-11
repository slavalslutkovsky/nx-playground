//! Error types for the notification service.

use std::fmt;

/// Result type for notification operations.
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Errors that can occur in the notification service.
#[derive(Debug)]
pub enum NotificationError {
    /// Redis queue operation failed
    QueueError(String),
    /// Serialization/deserialization error
    SerializationError(String),
    /// Configuration error
    ConfigError(String),
    /// Invalid input
    InvalidInput(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueError(msg) => write!(f, "Queue error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for NotificationError {}

impl From<redis::RedisError> for NotificationError {
    fn from(err: redis::RedisError) -> Self {
        Self::QueueError(err.to_string())
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}
