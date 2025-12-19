//! Worker configuration.
//!
//! This module provides configuration options for stream workers,
//! with sensible defaults and environment variable overrides.

use crate::registry::StreamDef;
use uuid::Uuid;

/// Configuration for stream workers.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    // Stream settings
    /// Redis stream name.
    pub stream_name: String,
    /// Consumer group name.
    pub consumer_group: String,
    /// Unique consumer/worker ID.
    pub consumer_id: String,

    // Processing settings
    /// Number of messages to read per batch.
    pub batch_size: usize,
    /// Poll interval in milliseconds (only used in polling mode).
    pub poll_interval_ms: u64,
    /// Block timeout in milliseconds for XREADGROUP BLOCK.
    /// - `None`: Use polling mode (check every poll_interval_ms)
    /// - `Some(1000)`: Block for 1 second (recommended for production)
    /// - `Some(0)`: Block indefinitely (not recommended - hard to shutdown)
    pub block_timeout_ms: Option<u64>,
    /// Maximum retry attempts before moving to DLQ.
    pub max_retries: u32,

    // Concurrency settings
    /// Maximum number of jobs to process concurrently.
    /// Set to 1 for sequential processing (default).
    /// Set higher for parallel processing of independent jobs.
    pub max_concurrent_jobs: usize,

    // DLQ settings
    /// Dead letter queue stream name.
    pub dlq_stream_name: String,
    /// Whether DLQ is enabled.
    pub enable_dlq: bool,

    // Recovery settings
    /// Time in seconds before claiming abandoned messages.
    pub claim_idle_time_secs: u64,
    /// Maximum stream length before auto-trim (MAXLEN).
    pub max_stream_length: i64,

    // Health settings
    /// Port for health check HTTP server.
    pub health_port: u16,
}

impl WorkerConfig {
    /// Create a new configuration with the given stream name.
    ///
    /// Uses sensible defaults for all other settings.
    pub fn new(stream_name: impl Into<String>) -> Self {
        let stream = stream_name.into();
        let domain = stream.split(':').next().unwrap_or(&stream);

        Self {
            stream_name: stream.clone(),
            consumer_group: format!("{}_workers", domain),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            batch_size: 10,
            poll_interval_ms: 500,
            block_timeout_ms: None, // Polling mode by default
            max_retries: 3,
            max_concurrent_jobs: 1, // Sequential by default for safety
            dlq_stream_name: format!("{}:dlq", domain),
            enable_dlq: true,
            claim_idle_time_secs: 5,
            max_stream_length: 100_000,
            health_port: 8081,
        }
    }

    /// Create configuration from a `StreamDef` implementation.
    ///
    /// This uses the stream definition constants for stream name,
    /// consumer group, and DLQ.
    pub fn from_stream_def<S: StreamDef>() -> Self {
        Self {
            stream_name: S::STREAM_NAME.to_string(),
            consumer_group: S::CONSUMER_GROUP.to_string(),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            batch_size: 10,
            poll_interval_ms: 500,
            block_timeout_ms: None, // Polling mode by default
            max_retries: 3,
            max_concurrent_jobs: 1, // Sequential by default for safety
            dlq_stream_name: S::DLQ_STREAM.to_string(),
            enable_dlq: true,
            claim_idle_time_secs: 5,
            max_stream_length: S::MAX_LENGTH,
            health_port: 8081,
        }
    }

    /// Create configuration from environment variables.
    ///
    /// Environment variables are prefixed with the given prefix (uppercase).
    /// For example, with prefix "EMAIL", it looks for:
    /// - `EMAIL_STREAM_NAME`
    /// - `EMAIL_CONSUMER_GROUP`
    /// - `EMAIL_POLL_INTERVAL_MS`
    /// - etc.
    pub fn from_env_with_prefix(prefix: &str) -> Self {
        let prefix = prefix.to_uppercase();

        let stream_name = std::env::var(format!("{}_STREAM_NAME", prefix))
            .unwrap_or_else(|_| format!("{}:jobs", prefix.to_lowercase()));

        let domain = stream_name.split(':').next().unwrap_or(&stream_name);

        Self {
            stream_name: stream_name.clone(),
            consumer_group: std::env::var(format!("{}_CONSUMER_GROUP", prefix))
                .unwrap_or_else(|_| format!("{}_workers", domain)),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            batch_size: std::env::var(format!("{}_BATCH_SIZE", prefix))
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            poll_interval_ms: std::env::var(format!("{}_POLL_INTERVAL_MS", prefix))
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),
            block_timeout_ms: std::env::var(format!("{}_BLOCK_TIMEOUT_MS", prefix))
                .ok()
                .and_then(|v| v.parse().ok()),
            max_retries: std::env::var(format!("{}_MAX_RETRIES", prefix))
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            max_concurrent_jobs: std::env::var(format!("{}_MAX_CONCURRENT_JOBS", prefix))
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            dlq_stream_name: std::env::var(format!("{}_DLQ_STREAM_NAME", prefix))
                .unwrap_or_else(|_| format!("{}:dlq", domain)),
            enable_dlq: std::env::var(format!("{}_ENABLE_DLQ", prefix))
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            claim_idle_time_secs: std::env::var(format!("{}_CLAIM_IDLE_TIME_SECS", prefix))
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_stream_length: std::env::var(format!("{}_MAX_STREAM_LENGTH", prefix))
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .unwrap_or(100_000),
            health_port: std::env::var("HEALTH_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .unwrap_or(8081),
        }
    }

    // Builder methods

    /// Set the batch size.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the poll interval in milliseconds (only used in polling mode).
    pub fn with_poll_interval_ms(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    /// Enable blocking mode with the given timeout in milliseconds.
    ///
    /// This is more efficient than polling as the worker waits on Redis
    /// for new messages instead of repeatedly checking.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - How long to block waiting for messages.
    ///   - `1000` (1 second) is recommended for production
    ///   - `0` blocks indefinitely (not recommended - hard to shutdown)
    ///
    /// # Example
    ///
    /// ```rust
    /// use stream_worker::WorkerConfig;
    ///
    /// let config = WorkerConfig::new("tasks:commands")
    ///     .with_blocking(1000)  // Block for 1 second
    ///     .with_batch_size(100)
    ///     .with_max_concurrent_jobs(20);
    /// ```
    pub fn with_blocking(mut self, timeout_ms: u64) -> Self {
        self.block_timeout_ms = Some(timeout_ms);
        self
    }

    /// Disable blocking mode (use polling instead).
    pub fn with_polling(mut self) -> Self {
        self.block_timeout_ms = None;
        self
    }

    /// Set the maximum retry count.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set a custom consumer ID.
    pub fn with_consumer_id(mut self, id: impl Into<String>) -> Self {
        self.consumer_id = id.into();
        self
    }

    /// Enable or disable DLQ.
    pub fn with_dlq(mut self, enabled: bool) -> Self {
        self.enable_dlq = enabled;
        self
    }

    /// Set the health port.
    pub fn with_health_port(mut self, port: u16) -> Self {
        self.health_port = port;
        self
    }

    /// Set the claim idle time in seconds.
    pub fn with_claim_idle_time_secs(mut self, secs: u64) -> Self {
        self.claim_idle_time_secs = secs;
        self
    }

    /// Set the maximum concurrent jobs.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum number of jobs to process concurrently.
    ///   - `1`: Sequential processing (default, safest)
    ///   - `>1`: Parallel processing (faster, requires idempotent jobs)
    ///
    /// # Example
    ///
    /// ```rust
    /// use stream_worker::WorkerConfig;
    ///
    /// let config = WorkerConfig::new("email:jobs")
    ///     .with_max_concurrent_jobs(10); // Process up to 10 emails concurrently
    /// ```
    pub fn with_max_concurrent_jobs(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max.max(1); // Ensure at least 1
        self
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self::new("default:jobs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = WorkerConfig::new("email:jobs");

        assert_eq!(config.stream_name, "email:jobs");
        assert_eq!(config.consumer_group, "email_workers");
        assert_eq!(config.dlq_stream_name, "email:dlq");
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.poll_interval_ms, 500);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_dlq);
    }

    #[test]
    fn test_config_builder() {
        let config = WorkerConfig::new("tasks:events")
            .with_batch_size(20)
            .with_poll_interval_ms(1000)
            .with_max_retries(5)
            .with_max_concurrent_jobs(10)
            .with_dlq(false);

        assert_eq!(config.batch_size, 20);
        assert_eq!(config.poll_interval_ms, 1000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.max_concurrent_jobs, 10);
        assert!(!config.enable_dlq);
    }

    #[test]
    fn test_config_concurrent_jobs_minimum() {
        // Ensure max_concurrent_jobs can't be set to 0
        let config = WorkerConfig::new("test:jobs").with_max_concurrent_jobs(0);
        assert_eq!(config.max_concurrent_jobs, 1);
    }

    struct TestStreamDef;
    impl StreamDef for TestStreamDef {
        const STREAM_NAME: &'static str = "test:stream";
        const CONSUMER_GROUP: &'static str = "test_workers";
        const DLQ_STREAM: &'static str = "test:dlq";
        const MAX_LENGTH: i64 = 50_000;
    }

    #[test]
    fn test_config_from_stream_def() {
        let config = WorkerConfig::from_stream_def::<TestStreamDef>();

        assert_eq!(config.stream_name, "test:stream");
        assert_eq!(config.consumer_group, "test_workers");
        assert_eq!(config.dlq_stream_name, "test:dlq");
        assert_eq!(config.max_stream_length, 50_000);
    }
}
