//! Error types for stream operations.
//!
//! This module provides:
//! - `StreamError` - The main error type for stream operations
//! - `ErrorCategory` - Classification of errors for smart retry logic
//! - `RetryStrategy` - How to handle retries based on error category

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during stream operations.
#[derive(Error, Debug)]
pub enum StreamError {
    /// Redis connection or command error.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Job processing error from the processor.
    #[error("Processing error: {0}")]
    Processing(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Stream/queue operation error.
    #[error("Queue error: {0}")]
    Queue(String),

    /// Consumer group error.
    #[error("Consumer group error: {0}")]
    ConsumerGroup(String),

    /// Job parsing error.
    #[error("Job parsing error: {0}")]
    JobParsing(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Health check failed.
    #[error("Health check failed: {0}")]
    HealthCheck(String),

    /// Timeout error.
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl StreamError {
    /// Check if this is a BLOCK timeout (normal behavior, not an error).
    ///
    /// When using XREADGROUP with BLOCK, a timeout means no messages arrived
    /// within the block period. This is expected behavior, not an error.
    pub fn is_block_timeout(&self) -> bool {
        match self {
            Self::Redis(e) => {
                let err_str = e.to_string().to_lowercase();
                // Redis BLOCK timeout - this is normal, not an error
                err_str.contains("timed out") && !err_str.contains("connection")
            }
            _ => false,
        }
    }

    /// Check if this is a connection-related error that might be recoverable.
    pub fn is_connection_error(&self) -> bool {
        match self {
            Self::Redis(e) => {
                let err_str = e.to_string().to_lowercase();
                // Don't treat BLOCK timeout as connection error
                if self.is_block_timeout() {
                    return false;
                }
                err_str.contains("connection")
                    || err_str.contains("disconnected")
                    || err_str.contains("broken pipe")
                    || err_str.contains("reset by peer")
                    || err_str.contains("refused")
                    || err_str.contains("eof")
                    || err_str.contains("io error")
            }
            Self::Queue(s) | Self::Internal(s) => {
                let lower = s.to_lowercase();
                lower.contains("connection") && !lower.contains("block")
            }
            _ => false,
        }
    }

    /// Check if this is a consumer group missing error (NOGROUP).
    pub fn is_nogroup_error(&self) -> bool {
        match self {
            Self::Redis(e) => e.to_string().contains("NOGROUP"),
            Self::ConsumerGroup(s) => s.contains("NOGROUP"),
            Self::Queue(s) => s.contains("NOGROUP"),
            _ => false,
        }
    }

    /// Check if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        match self {
            Self::Redis(e) => {
                let err_str = e.to_string().to_lowercase();
                err_str.contains("timeout") || err_str.contains("timed out")
            }
            Self::Timeout(_) => true,
            _ => false,
        }
    }

    /// Categorize the error for smart retry logic.
    pub fn category(&self) -> ErrorCategory {
        match self {
            // Connection errors are transient - retry with backoff
            Self::Redis(e) => {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("connection")
                    || err_str.contains("disconnected")
                    || err_str.contains("broken pipe")
                    || err_str.contains("reset by peer")
                    || err_str.contains("refused")
                    || err_str.contains("eof")
                    || err_str.contains("io error")
                {
                    ErrorCategory::Transient
                } else if err_str.contains("timeout") || err_str.contains("timed out") {
                    ErrorCategory::Transient
                } else if err_str.contains("busy") || err_str.contains("loading") {
                    ErrorCategory::RateLimited
                } else {
                    ErrorCategory::Transient
                }
            }

            // Queue/internal errors are usually transient
            Self::Queue(s) | Self::Internal(s) => {
                let lower = s.to_lowercase();
                if lower.contains("connection") || lower.contains("timeout") {
                    ErrorCategory::Transient
                } else if lower.contains("rate") || lower.contains("limit") || lower.contains("quota") {
                    ErrorCategory::RateLimited
                } else {
                    ErrorCategory::Transient
                }
            }

            // Timeout is transient
            Self::Timeout(_) => ErrorCategory::Transient,

            // Consumer group errors are transient (can be recreated)
            Self::ConsumerGroup(_) => ErrorCategory::Transient,

            // Health check failures are transient
            Self::HealthCheck(_) => ErrorCategory::Transient,

            // Serialization errors are permanent - bad data, don't retry
            Self::Serialization(_) => ErrorCategory::Permanent,

            // Job parsing errors are permanent - malformed message
            Self::JobParsing(_) => ErrorCategory::Permanent,

            // Config errors are permanent
            Self::Config(_) => ErrorCategory::Permanent,

            // Processing errors need inspection
            Self::Processing(s) => {
                let lower = s.to_lowercase();
                // Rate limiting indicators
                if lower.contains("rate")
                    || lower.contains("limit")
                    || lower.contains("quota")
                    || lower.contains("throttl")
                    || lower.contains("429")
                    || lower.contains("too many")
                {
                    ErrorCategory::RateLimited
                }
                // Permanent failure indicators
                else if lower.contains("invalid")
                    || lower.contains("malformed")
                    || lower.contains("not found")
                    || lower.contains("does not exist")
                    || lower.contains("forbidden")
                    || lower.contains("unauthorized")
                    || lower.contains("401")
                    || lower.contains("403")
                    || lower.contains("404")
                {
                    ErrorCategory::Permanent
                }
                // Transient indicators
                else if lower.contains("timeout")
                    || lower.contains("connection")
                    || lower.contains("temporarily")
                    || lower.contains("unavailable")
                    || lower.contains("500")
                    || lower.contains("502")
                    || lower.contains("503")
                    || lower.contains("504")
                {
                    ErrorCategory::Transient
                }
                // Default to transient for unknown processing errors
                else {
                    ErrorCategory::Transient
                }
            }
        }
    }
}

/// Error category for smart retry logic.
///
/// Different error categories have different retry strategies:
/// - `Transient`: Temporary issues that will likely resolve with retry (connection issues, timeouts)
/// - `Permanent`: Errors that won't be fixed by retrying (invalid data, auth failures)
/// - `RateLimited`: Service is overloaded, need longer backoff
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Temporary error - retry with exponential backoff.
    /// Examples: connection timeout, service temporarily unavailable.
    Transient,

    /// Permanent error - do not retry, move to DLQ immediately.
    /// Examples: invalid data format, authentication failure, resource not found.
    Permanent,

    /// Rate limited - retry with longer backoff.
    /// Examples: 429 Too Many Requests, quota exceeded.
    RateLimited,
}

impl ErrorCategory {
    /// Get the retry strategy for this error category.
    pub fn retry_strategy(&self) -> RetryStrategy {
        match self {
            Self::Transient => RetryStrategy::ExponentialBackoff {
                base_delay_ms: 1000,
                max_delay_ms: 30_000,
                max_retries: 3,
            },
            Self::Permanent => RetryStrategy::NoRetry,
            Self::RateLimited => RetryStrategy::ExponentialBackoff {
                base_delay_ms: 5000,
                max_delay_ms: 120_000, // Up to 2 minutes
                max_retries: 5,        // More retries for rate limiting
            },
        }
    }

    /// Check if this error category should be retried.
    pub fn should_retry(&self) -> bool {
        !matches!(self, Self::Permanent)
    }
}

/// Retry strategy for handling errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryStrategy {
    /// Do not retry, move to DLQ immediately.
    NoRetry,

    /// Retry with exponential backoff.
    ExponentialBackoff {
        /// Base delay in milliseconds.
        base_delay_ms: u64,
        /// Maximum delay in milliseconds.
        max_delay_ms: u64,
        /// Maximum number of retries.
        max_retries: u32,
    },
}

impl RetryStrategy {
    /// Calculate the delay for a given retry attempt.
    ///
    /// Uses exponential backoff with jitter to prevent thundering herd.
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            Self::NoRetry => None,
            Self::ExponentialBackoff {
                base_delay_ms,
                max_delay_ms,
                max_retries,
            } => {
                if attempt >= *max_retries {
                    return None;
                }

                // Calculate base exponential delay
                let exp_delay = base_delay_ms.saturating_mul(2u64.pow(attempt));
                let capped_delay = exp_delay.min(*max_delay_ms);

                // Apply jitter (±25% randomness)
                let final_delay = Self::apply_jitter(capped_delay);

                Some(Duration::from_millis(final_delay))
            }
        }
    }

    /// Apply jitter to a delay (±25% randomness).
    ///
    /// Returns the delay with jitter applied, keeping it within ±25% of the original.
    fn apply_jitter(delay_ms: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        // Simple pseudo-random based on current time
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .hash(&mut hasher);
        let hash = hasher.finish();

        // Calculate jitter range as ±25% of delay
        let jitter_range = delay_ms / 4;
        if jitter_range == 0 {
            return delay_ms;
        }

        // Map hash to range [0, 2*jitter_range] then shift to [-jitter_range, +jitter_range]
        let random_offset = (hash % (jitter_range * 2 + 1)) as i64 - jitter_range as i64;

        if random_offset < 0 {
            delay_ms.saturating_sub((-random_offset) as u64)
        } else {
            delay_ms.saturating_add(random_offset as u64)
        }
    }

    /// Check if we should retry for a given attempt number.
    pub fn should_retry(&self, attempt: u32) -> bool {
        match self {
            Self::NoRetry => false,
            Self::ExponentialBackoff { max_retries, .. } => attempt < *max_retries,
        }
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> u32 {
        match self {
            Self::NoRetry => 0,
            Self::ExponentialBackoff { max_retries, .. } => *max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = StreamError::Processing("failed to send email".to_string());
        assert_eq!(err.to_string(), "Processing error: failed to send email");
    }

    #[test]
    fn test_is_connection_error() {
        let err = StreamError::Queue("connection refused".to_string());
        assert!(err.is_connection_error());

        let err = StreamError::Processing("invalid data".to_string());
        assert!(!err.is_connection_error());
    }

    #[test]
    fn test_error_category_transient() {
        let err = StreamError::Timeout("operation timed out".to_string());
        assert_eq!(err.category(), ErrorCategory::Transient);

        let err = StreamError::Queue("connection reset".to_string());
        assert_eq!(err.category(), ErrorCategory::Transient);

        let err = StreamError::Processing("503 service unavailable".to_string());
        assert_eq!(err.category(), ErrorCategory::Transient);
    }

    #[test]
    fn test_error_category_permanent() {
        let err = StreamError::JobParsing("invalid json".to_string());
        assert_eq!(err.category(), ErrorCategory::Permanent);

        let err = StreamError::Config("missing required field".to_string());
        assert_eq!(err.category(), ErrorCategory::Permanent);

        let err = StreamError::Processing("404 not found".to_string());
        assert_eq!(err.category(), ErrorCategory::Permanent);

        let err = StreamError::Processing("unauthorized access".to_string());
        assert_eq!(err.category(), ErrorCategory::Permanent);
    }

    #[test]
    fn test_error_category_rate_limited() {
        let err = StreamError::Processing("429 too many requests".to_string());
        assert_eq!(err.category(), ErrorCategory::RateLimited);

        let err = StreamError::Processing("rate limit exceeded".to_string());
        assert_eq!(err.category(), ErrorCategory::RateLimited);

        let err = StreamError::Queue("quota exceeded".to_string());
        assert_eq!(err.category(), ErrorCategory::RateLimited);
    }

    #[test]
    fn test_retry_strategy_no_retry() {
        let strategy = RetryStrategy::NoRetry;
        assert!(!strategy.should_retry(0));
        assert_eq!(strategy.delay_for_attempt(0), None);
        assert_eq!(strategy.max_retries(), 0);
    }

    #[test]
    fn test_retry_strategy_exponential_backoff() {
        let strategy = RetryStrategy::ExponentialBackoff {
            base_delay_ms: 1000,
            max_delay_ms: 30_000,
            max_retries: 3,
        };

        // Should retry for attempts 0, 1, 2
        assert!(strategy.should_retry(0));
        assert!(strategy.should_retry(1));
        assert!(strategy.should_retry(2));
        assert!(!strategy.should_retry(3));

        // Delays should exist for attempts 0, 1, 2
        assert!(strategy.delay_for_attempt(0).is_some());
        assert!(strategy.delay_for_attempt(1).is_some());
        assert!(strategy.delay_for_attempt(2).is_some());
        assert!(strategy.delay_for_attempt(3).is_none());

        assert_eq!(strategy.max_retries(), 3);
    }

    #[test]
    fn test_retry_strategy_delay_capping() {
        let strategy = RetryStrategy::ExponentialBackoff {
            base_delay_ms: 1000,
            max_delay_ms: 5000,
            max_retries: 10,
        };

        // High attempt numbers should be capped at max_delay
        let delay = strategy.delay_for_attempt(8).unwrap();
        // With jitter ±25%, max should be around 5000 + 1250 = 6250
        assert!(delay.as_millis() <= 7000);
    }

    #[test]
    fn test_error_category_should_retry() {
        assert!(ErrorCategory::Transient.should_retry());
        assert!(!ErrorCategory::Permanent.should_retry());
        assert!(ErrorCategory::RateLimited.should_retry());
    }

    #[test]
    fn test_error_category_retry_strategy() {
        // Transient errors have standard backoff
        let strategy = ErrorCategory::Transient.retry_strategy();
        assert!(matches!(strategy, RetryStrategy::ExponentialBackoff { max_retries: 3, .. }));

        // Permanent errors don't retry
        let strategy = ErrorCategory::Permanent.retry_strategy();
        assert!(matches!(strategy, RetryStrategy::NoRetry));

        // Rate limited errors have longer backoff with more retries
        let strategy = ErrorCategory::RateLimited.retry_strategy();
        assert!(matches!(strategy, RetryStrategy::ExponentialBackoff { max_retries: 5, .. }));
    }
}
