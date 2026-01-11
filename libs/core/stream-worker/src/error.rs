//! Stream error types and error categorization
//!
//! Errors are categorized to determine retry behavior:
//! - **Transient**: Temporary failures, retry with exponential backoff
//! - **Permanent**: Unrecoverable errors, move to DLQ immediately
//! - **RateLimited**: Rate limit hit, longer backoff before retry

use thiserror::Error;

/// Category of error for determining retry behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Temporary failure - retry with exponential backoff (1s-30s, 3 retries)
    Transient,
    /// Unrecoverable error - move to DLQ immediately
    Permanent,
    /// Rate limit hit - longer backoff (5s-120s, 5 retries)
    RateLimited,
}

impl ErrorCategory {
    /// Get the maximum retries for this error category
    pub fn max_retries(&self) -> u32 {
        match self {
            ErrorCategory::Transient => 3,
            ErrorCategory::Permanent => 0,
            ErrorCategory::RateLimited => 5,
        }
    }

    /// Get the base delay in milliseconds for this error category
    pub fn base_delay_ms(&self) -> u64 {
        match self {
            ErrorCategory::Transient => 1000,   // 1 second
            ErrorCategory::Permanent => 0,      // No retry
            ErrorCategory::RateLimited => 5000, // 5 seconds
        }
    }

    /// Get the maximum delay in milliseconds for this error category
    pub fn max_delay_ms(&self) -> u64 {
        match self {
            ErrorCategory::Transient => 30_000,    // 30 seconds
            ErrorCategory::Permanent => 0,         // No retry
            ErrorCategory::RateLimited => 120_000, // 2 minutes
        }
    }

    /// Calculate exponential backoff delay for given retry count
    pub fn backoff_delay_ms(&self, retry_count: u32) -> u64 {
        if *self == ErrorCategory::Permanent {
            return 0;
        }

        let base = self.base_delay_ms();
        let max = self.max_delay_ms();
        let delay = base * 2u64.saturating_pow(retry_count);
        delay.min(max)
    }
}

/// Stream processing errors
#[derive(Error, Debug)]
pub enum StreamError {
    /// Redis connection or command error
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Job processing failed
    #[error("Processing error: {message}")]
    Processing {
        message: String,
        category: ErrorCategory,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Circuit breaker open
    #[error("Circuit breaker open: {0}")]
    CircuitOpen(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Shutdown requested
    #[error("Shutdown requested")]
    Shutdown,
}

impl StreamError {
    /// Create a transient processing error
    pub fn transient(message: impl Into<String>) -> Self {
        StreamError::Processing {
            message: message.into(),
            category: ErrorCategory::Transient,
        }
    }

    /// Create a permanent processing error
    pub fn permanent(message: impl Into<String>) -> Self {
        StreamError::Processing {
            message: message.into(),
            category: ErrorCategory::Permanent,
        }
    }

    /// Create a rate limited error
    pub fn rate_limited(message: impl Into<String>) -> Self {
        StreamError::RateLimited(message.into())
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            StreamError::Redis(_) => ErrorCategory::Transient,
            StreamError::Serialization(_) => ErrorCategory::Permanent,
            StreamError::Processing { category, .. } => *category,
            StreamError::RateLimited(_) => ErrorCategory::RateLimited,
            StreamError::CircuitOpen(_) => ErrorCategory::Transient,
            StreamError::Config(_) => ErrorCategory::Permanent,
            StreamError::Internal(_) => ErrorCategory::Permanent,
            StreamError::Shutdown => ErrorCategory::Permanent,
        }
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self, retry_count: u32) -> bool {
        let category = self.category();
        category != ErrorCategory::Permanent && retry_count < category.max_retries()
    }

    /// Get the backoff delay for retry
    pub fn backoff_delay_ms(&self, retry_count: u32) -> u64 {
        self.category().backoff_delay_ms(retry_count)
    }
}

impl From<serde_json::Error> for StreamError {
    fn from(err: serde_json::Error) -> Self {
        StreamError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        assert_eq!(ErrorCategory::Transient.max_retries(), 3);
        assert_eq!(ErrorCategory::Permanent.max_retries(), 0);
        assert_eq!(ErrorCategory::RateLimited.max_retries(), 5);
    }

    #[test]
    fn test_backoff_calculation() {
        // Transient: 1s, 2s, 4s, 8s, 16s, 30s (max)
        assert_eq!(ErrorCategory::Transient.backoff_delay_ms(0), 1000);
        assert_eq!(ErrorCategory::Transient.backoff_delay_ms(1), 2000);
        assert_eq!(ErrorCategory::Transient.backoff_delay_ms(2), 4000);
        assert_eq!(ErrorCategory::Transient.backoff_delay_ms(5), 30_000); // Capped at max

        // RateLimited: 5s, 10s, 20s, 40s, 80s, 120s (max)
        assert_eq!(ErrorCategory::RateLimited.backoff_delay_ms(0), 5000);
        assert_eq!(ErrorCategory::RateLimited.backoff_delay_ms(1), 10_000);
        assert_eq!(ErrorCategory::RateLimited.backoff_delay_ms(5), 120_000); // Capped at max

        // Permanent: no retry
        assert_eq!(ErrorCategory::Permanent.backoff_delay_ms(0), 0);
    }

    #[test]
    fn test_should_retry() {
        let transient = StreamError::transient("test");
        assert!(transient.should_retry(0));
        assert!(transient.should_retry(2));
        assert!(!transient.should_retry(3));

        let permanent = StreamError::permanent("test");
        assert!(!permanent.should_retry(0));
    }
}
