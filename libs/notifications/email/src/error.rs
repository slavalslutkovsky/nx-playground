//! Error types for the notification service.
//!
//! IMPROVEMENT: Removed Redis dependency - this is now a NATS-only library.

use std::fmt;

/// Result type for notification operations.
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Errors that can occur in the notification service.
#[derive(Debug)]
pub enum NotificationError {
    /// NATS queue operation failed
    QueueError(String),
    /// Serialization/deserialization error
    SerializationError(String),
    /// Configuration error
    ConfigError(String),
    /// Invalid input
    InvalidInput(String),
    /// Provider error (SMTP, SendGrid, etc.)
    ProviderError(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueError(msg) => write!(f, "Queue error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::ProviderError(msg) => write!(f, "Provider error: {}", msg),
        }
    }
}

impl std::error::Error for NotificationError {}

impl From<serde_json::Error> for NotificationError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<eyre::Report> for NotificationError {
    fn from(err: eyre::Report) -> Self {
        Self::ProviderError(err.to_string())
    }
}
