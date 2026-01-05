//! Worker configuration
//!
//! This module provides `WorkerConfig` for configuring the stream worker.

use crate::registry::StreamDef;
use uuid::Uuid;

/// Configuration for the stream worker
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Redis stream name
    pub stream_name: String,

    /// Consumer group name
    pub consumer_group: String,

    /// Unique consumer ID (auto-generated if not provided)
    pub consumer_id: String,

    /// Dead letter queue stream name
    pub dlq_stream: String,

    /// Maximum stream length before trimming
    pub max_length: i64,

    /// Poll interval in milliseconds when no messages available
    pub poll_interval_ms: u64,

    /// Batch size for reading messages
    pub batch_size: usize,

    /// Blocking read timeout in milliseconds (None = non-blocking)
    pub blocking_timeout_ms: Option<u64>,

    /// Maximum concurrent jobs to process
    pub max_concurrent_jobs: usize,

    /// Claim timeout in milliseconds for abandoned messages
    pub claim_timeout_ms: u64,

    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,

    /// Enable rate limiting
    pub enable_rate_limiter: bool,

    /// Rate limit: max requests per second
    pub rate_limit_rps: f64,
}

impl WorkerConfig {
    /// Create a new WorkerConfig from a StreamDef
    pub fn from_stream_def<S: StreamDef>() -> Self {
        Self {
            stream_name: S::STREAM_NAME.to_string(),
            consumer_group: S::CONSUMER_GROUP.to_string(),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            dlq_stream: S::DLQ_STREAM.to_string(),
            max_length: S::MAX_LENGTH,
            poll_interval_ms: S::POLL_INTERVAL_MS,
            batch_size: S::BATCH_SIZE,
            blocking_timeout_ms: Some(5000), // Default 5s blocking
            max_concurrent_jobs: 1,           // Sequential by default
            claim_timeout_ms: S::CLAIM_TIMEOUT_MS,
            enable_circuit_breaker: true,
            enable_rate_limiter: false,
            rate_limit_rps: 100.0,
        }
    }

    /// Create a new WorkerConfig with explicit values
    pub fn new(stream_name: impl Into<String>, consumer_group: impl Into<String>) -> Self {
        Self {
            stream_name: stream_name.into(),
            consumer_group: consumer_group.into(),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            dlq_stream: String::new(),
            max_length: 100_000,
            poll_interval_ms: 1000,
            batch_size: 10,
            blocking_timeout_ms: Some(5000),
            max_concurrent_jobs: 1,
            claim_timeout_ms: 30_000,
            enable_circuit_breaker: true,
            enable_rate_limiter: false,
            rate_limit_rps: 100.0,
        }
    }

    /// Set the consumer ID
    pub fn with_consumer_id(mut self, id: impl Into<String>) -> Self {
        self.consumer_id = id.into();
        self
    }

    /// Set the DLQ stream name
    pub fn with_dlq_stream(mut self, stream: impl Into<String>) -> Self {
        self.dlq_stream = stream.into();
        self
    }

    /// Set the maximum stream length
    pub fn with_max_length(mut self, max_length: i64) -> Self {
        self.max_length = max_length;
        self
    }

    /// Set the poll interval
    pub fn with_poll_interval_ms(mut self, interval: u64) -> Self {
        self.poll_interval_ms = interval;
        self
    }

    /// Set the batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the blocking timeout (None for non-blocking)
    pub fn with_blocking(mut self, timeout_ms: Option<u64>) -> Self {
        self.blocking_timeout_ms = timeout_ms;
        self
    }

    /// Set the maximum concurrent jobs
    pub fn with_max_concurrent_jobs(mut self, count: usize) -> Self {
        self.max_concurrent_jobs = count.max(1);
        self
    }

    /// Set the claim timeout for abandoned messages
    pub fn with_claim_timeout_ms(mut self, timeout: u64) -> Self {
        self.claim_timeout_ms = timeout;
        self
    }

    /// Enable or disable circuit breaker
    pub fn with_circuit_breaker(mut self, enable: bool) -> Self {
        self.enable_circuit_breaker = enable;
        self
    }

    /// Enable or disable rate limiting
    pub fn with_rate_limiter(mut self, enable: bool, rps: f64) -> Self {
        self.enable_rate_limiter = enable;
        self.rate_limit_rps = rps;
        self
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self::new("stream:jobs", "workers")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStream;

    impl StreamDef for TestStream {
        const STREAM_NAME: &'static str = "test:stream";
        const CONSUMER_GROUP: &'static str = "test:group";
        const DLQ_STREAM: &'static str = "test:dlq";
    }

    #[test]
    fn test_from_stream_def() {
        let config = WorkerConfig::from_stream_def::<TestStream>();

        assert_eq!(config.stream_name, "test:stream");
        assert_eq!(config.consumer_group, "test:group");
        assert_eq!(config.dlq_stream, "test:dlq");
        assert!(config.consumer_id.starts_with("worker-"));
    }

    #[test]
    fn test_builder_pattern() {
        let config = WorkerConfig::new("my:stream", "my:group")
            .with_consumer_id("worker-1")
            .with_dlq_stream("my:dlq")
            .with_batch_size(20)
            .with_max_concurrent_jobs(4)
            .with_blocking(Some(10_000));

        assert_eq!(config.stream_name, "my:stream");
        assert_eq!(config.consumer_id, "worker-1");
        assert_eq!(config.batch_size, 20);
        assert_eq!(config.max_concurrent_jobs, 4);
        assert_eq!(config.blocking_timeout_ms, Some(10_000));
    }
}
