//! Core traits for stream processing
//!
//! This module defines the core abstractions:
//! - `StreamJob`: A job that can be processed from a stream
//! - `StreamDef`: Stream configuration (type-safe constants)
//! - `StreamProcessor`: Job processor trait

use crate::StreamError;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// A job that can be processed from a Redis stream
///
/// Implement this trait for your job types to make them processable
/// by the `StreamWorker`.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Serialize, Deserialize)]
/// struct EmailJob {
///     id: Uuid,
///     to: String,
///     subject: String,
///     retry_count: u32,
/// }
///
/// impl StreamJob for EmailJob {
///     fn job_id(&self) -> String { self.id.to_string() }
///     fn retry_count(&self) -> u32 { self.retry_count }
///     fn with_retry(&self) -> Self {
///         Self { retry_count: self.retry_count + 1, ..self.clone() }
///     }
/// }
/// ```
pub trait StreamJob: Serialize + DeserializeOwned + Send + Sync + Clone + 'static {
    /// Get the unique job ID
    fn job_id(&self) -> String;

    /// Get the current retry count
    fn retry_count(&self) -> u32;

    /// Create a new instance with incremented retry count
    fn with_retry(&self) -> Self;

    /// Get the maximum number of retries (default: 3)
    fn max_retries(&self) -> u32 {
        3
    }

    /// Check if the job can be retried
    fn can_retry(&self) -> bool {
        self.retry_count() < self.max_retries()
    }
}

/// Stream configuration (type-safe constants)
///
/// Implement this trait to define your stream's Redis keys and settings.
///
/// # Example
///
/// ```ignore
/// struct EmailStream;
///
/// impl StreamDef for EmailStream {
///     const STREAM_NAME: &'static str = "email:jobs";
///     const CONSUMER_GROUP: &'static str = "email_workers";
///     const DLQ_STREAM: &'static str = "email:dlq";
///     const MAX_LENGTH: i64 = 100_000;
/// }
/// ```
pub trait StreamDef {
    /// The Redis stream name (e.g., "email:jobs")
    const STREAM_NAME: &'static str;

    /// The consumer group name (e.g., "email_workers")
    const CONSUMER_GROUP: &'static str;

    /// The dead letter queue stream name (e.g., "email:dlq")
    const DLQ_STREAM: &'static str;

    /// Maximum stream length before trimming (default: 100,000)
    const MAX_LENGTH: i64 = 100_000;

    /// Default poll interval in milliseconds (default: 1000)
    const POLL_INTERVAL_MS: u64 = 1000;

    /// Default batch size for reading messages (default: 10)
    const BATCH_SIZE: usize = 10;

    /// Claim timeout in milliseconds for abandoned messages (default: 30,000)
    const CLAIM_TIMEOUT_MS: u64 = 30_000;
}

/// Job processor trait
///
/// Implement this trait to define how jobs are processed.
///
/// # Example
///
/// ```ignore
/// struct EmailProcessor {
///     provider: Arc<dyn EmailProvider>,
///     templates: Arc<TemplateEngine>,
/// }
///
/// #[async_trait]
/// impl StreamProcessor<EmailJob> for EmailProcessor {
///     async fn process(&self, job: &EmailJob) -> Result<(), StreamError> {
///         let rendered = self.templates.render(&job.template, &job.data)?;
///         self.provider.send(&rendered).await?;
///         Ok(())
///     }
///
///     fn name(&self) -> &'static str {
///         "email_processor"
///     }
/// }
/// ```
#[async_trait]
pub trait StreamProcessor<J: StreamJob>: Send + Sync {
    /// Process a job
    ///
    /// Return `Ok(())` on success, or a `StreamError` on failure.
    /// The error category determines retry behavior.
    async fn process(&self, job: &J) -> Result<(), StreamError>;

    /// Get the processor name (for logging and metrics)
    fn name(&self) -> &'static str;

    /// Perform a health check
    ///
    /// Override to check downstream service availability.
    async fn health_check(&self) -> Result<bool, StreamError> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    struct TestJob {
        id: String,
        retry_count: u32,
    }

    impl StreamJob for TestJob {
        fn job_id(&self) -> String {
            self.id.clone()
        }

        fn retry_count(&self) -> u32 {
            self.retry_count
        }

        fn with_retry(&self) -> Self {
            Self {
                id: self.id.clone(),
                retry_count: self.retry_count + 1,
            }
        }
    }

    struct TestStream;

    impl StreamDef for TestStream {
        const STREAM_NAME: &'static str = "test:jobs";
        const CONSUMER_GROUP: &'static str = "test_workers";
        const DLQ_STREAM: &'static str = "test:dlq";
    }

    #[test]
    fn test_stream_job() {
        let job = TestJob {
            id: "job-1".to_string(),
            retry_count: 0,
        };

        assert_eq!(job.job_id(), "job-1");
        assert_eq!(job.retry_count(), 0);
        assert!(job.can_retry());

        let retried = job.with_retry();
        assert_eq!(retried.retry_count(), 1);
    }

    #[test]
    fn test_stream_def() {
        assert_eq!(TestStream::STREAM_NAME, "test:jobs");
        assert_eq!(TestStream::CONSUMER_GROUP, "test_workers");
        assert_eq!(TestStream::DLQ_STREAM, "test:dlq");
        assert_eq!(TestStream::MAX_LENGTH, 100_000);
    }
}
