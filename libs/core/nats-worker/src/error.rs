//! Error types for NATS worker.

use messaging::ErrorCategory;
use thiserror::Error;

/// Error that can occur in NATS worker operations.
#[derive(Debug, Error)]
pub enum NatsError {
    /// NATS connection error
    #[error("NATS connection error: {0}")]
    Connection(#[from] async_nats::ConnectError),

    /// JetStream error
    #[error("JetStream error: {0}")]
    JetStream(String),

    /// Consumer error
    #[error("Consumer error: {0}")]
    Consumer(String),

    /// Publish error
    #[error("Publish error: {0}")]
    Publish(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Processing error
    #[error("Processing error: {0}")]
    Processing(#[from] messaging::ProcessingError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Stream not found
    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    /// Consumer not found
    #[error("Consumer not found: {0}")]
    ConsumerNotFound(String),
}

impl NatsError {
    /// Get the error category for retry decisions.
    pub fn category(&self) -> ErrorCategory {
        match self {
            // Transient errors - should retry
            NatsError::Connection(_) => ErrorCategory::Transient,
            NatsError::Timeout(_) => ErrorCategory::Transient,
            NatsError::JetStream(msg) if msg.contains("timeout") => ErrorCategory::Transient,
            NatsError::Publish(msg) if msg.contains("timeout") => ErrorCategory::Transient,

            // Permanent errors - don't retry
            NatsError::Serialization(_) => ErrorCategory::Permanent,
            NatsError::Config(_) => ErrorCategory::Permanent,
            NatsError::StreamNotFound(_) => ErrorCategory::Permanent,
            NatsError::ConsumerNotFound(_) => ErrorCategory::Permanent,

            // Processing errors - delegate to inner category
            NatsError::Processing(e) => e.category(),

            // Default to transient
            _ => ErrorCategory::Transient,
        }
    }

    /// Check if this error should be retried.
    pub fn should_retry(&self, retry_count: u32) -> bool {
        self.category().should_retry(retry_count)
    }

    /// Calculate backoff delay for retry.
    pub fn backoff_delay_ms(&self, retry_count: u32) -> u64 {
        self.category().backoff_delay_ms(retry_count)
    }

    /// Create a JetStream error from an async_nats error.
    pub fn from_jetstream_error(error: impl std::fmt::Display) -> Self {
        Self::JetStream(error.to_string())
    }

    /// Create a publish error.
    pub fn publish_error(msg: impl Into<String>) -> Self {
        Self::Publish(msg.into())
    }

    /// Create a consumer error.
    pub fn consumer_error(msg: impl Into<String>) -> Self {
        Self::Consumer(msg.into())
    }
}

// Note: async_nats::error::Error requires specific handling per error type
// Use NatsError::from_jetstream_error(e) for conversion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category() {
        let timeout_err = NatsError::Timeout("timed out".to_string());
        assert_eq!(timeout_err.category(), ErrorCategory::Transient);

        let serialization_err =
            NatsError::Serialization(serde_json::from_str::<String>("invalid").unwrap_err());
        assert_eq!(serialization_err.category(), ErrorCategory::Permanent);

        let config_err = NatsError::Config("bad config".to_string());
        assert_eq!(config_err.category(), ErrorCategory::Permanent);
    }

    #[test]
    fn test_should_retry() {
        let transient = NatsError::Timeout("timed out".to_string());
        assert!(transient.should_retry(0));
        assert!(transient.should_retry(2));
        assert!(!transient.should_retry(3));

        let permanent = NatsError::Config("invalid".to_string());
        assert!(!permanent.should_retry(0));
    }
}
