//! Metrics module for stream worker observability.
//!
//! This module provides Prometheus-compatible metrics for monitoring
//! stream worker performance and health.
//!
//! ## Available Metrics
//!
//! - `stream_jobs_processed_total` - Counter of jobs processed by status
//! - `stream_job_processing_duration_seconds` - Histogram of job processing time
//! - `stream_job_queue_depth` - Gauge of current queue depth
//! - `stream_errors_total` - Counter of errors by type
//! - `stream_retries_total` - Counter of retry attempts
//! - `stream_consumer_lag` - Gauge of consumer group lag

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;
use std::time::Duration;

/// Global Prometheus handle for metrics export
static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Metric names as constants for consistency
pub mod names {
    pub const JOBS_PROCESSED: &str = "stream_jobs_processed_total";
    pub const JOB_DURATION: &str = "stream_job_processing_duration_seconds";
    pub const QUEUE_DEPTH: &str = "stream_job_queue_depth";
    pub const ERRORS: &str = "stream_errors_total";
    pub const RETRIES: &str = "stream_retries_total";
    pub const CONSUMER_LAG: &str = "stream_consumer_lag";
    pub const BATCH_SIZE: &str = "stream_batch_size";
    pub const DLQ_SIZE: &str = "stream_dlq_size";
    /// Current number of jobs being processed concurrently
    pub const IN_FLIGHT_JOBS: &str = "stream_in_flight_jobs";
}

/// Job processing status for metrics labeling
#[derive(Debug, Clone, Copy)]
pub enum JobStatus {
    Success,
    Failed,
    Dlq,
    Skipped,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Dlq => "dlq",
            Self::Skipped => "skipped",
        }
    }
}

/// Error category for metrics labeling
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Transient,
    Permanent,
    RateLimited,
    Validation,
    Connection,
    Timeout,
    Unknown,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::Permanent => "permanent",
            Self::RateLimited => "rate_limited",
            Self::Validation => "validation",
            Self::Connection => "connection",
            Self::Timeout => "timeout",
            Self::Unknown => "unknown",
        }
    }
}

/// Initialize the Prometheus metrics exporter.
///
/// This should be called once at application startup.
/// Returns the PrometheusHandle for rendering metrics.
///
/// # Example
///
/// ```ignore
/// use stream_worker::metrics::init_metrics;
///
/// let handle = init_metrics();
/// // Use handle.render() to get Prometheus text format
/// ```
pub fn init_metrics() -> PrometheusHandle {
    PROMETHEUS_HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("Failed to install Prometheus recorder")
        })
        .clone()
}

/// Get the global Prometheus handle.
///
/// Returns None if metrics haven't been initialized.
pub fn get_metrics_handle() -> Option<PrometheusHandle> {
    PROMETHEUS_HANDLE.get().cloned()
}

/// Record a job as processed with the given status.
pub fn record_job_processed(stream: &str, status: JobStatus) {
    counter!(
        names::JOBS_PROCESSED,
        "stream" => stream.to_string(),
        "status" => status.as_str().to_string()
    )
    .increment(1);
}

/// Record job processing duration.
pub fn record_job_duration(stream: &str, operation: &str, duration: Duration) {
    histogram!(
        names::JOB_DURATION,
        "stream" => stream.to_string(),
        "operation" => operation.to_string()
    )
    .record(duration.as_secs_f64());
}

/// Update the queue depth gauge.
pub fn set_queue_depth(stream: &str, depth: f64) {
    gauge!(
        names::QUEUE_DEPTH,
        "stream" => stream.to_string()
    )
    .set(depth);
}

/// Record an error by type.
pub fn record_error(stream: &str, error_type: ErrorType) {
    counter!(
        names::ERRORS,
        "stream" => stream.to_string(),
        "error_type" => error_type.as_str().to_string()
    )
    .increment(1);
}

/// Record a retry attempt.
pub fn record_retry(stream: &str, attempt: u32) {
    counter!(
        names::RETRIES,
        "stream" => stream.to_string(),
        "attempt" => attempt.to_string()
    )
    .increment(1);
}

/// Update the consumer lag gauge.
pub fn set_consumer_lag(stream: &str, consumer_group: &str, lag: f64) {
    gauge!(
        names::CONSUMER_LAG,
        "stream" => stream.to_string(),
        "consumer_group" => consumer_group.to_string()
    )
    .set(lag);
}

/// Record the batch size processed.
pub fn record_batch_size(stream: &str, size: usize) {
    histogram!(
        names::BATCH_SIZE,
        "stream" => stream.to_string()
    )
    .record(size as f64);
}

/// Update the DLQ size gauge.
pub fn set_dlq_size(stream: &str, size: f64) {
    gauge!(
        names::DLQ_SIZE,
        "stream" => stream.to_string()
    )
    .set(size);
}

/// Update the in-flight jobs gauge (concurrent processing).
///
/// This tracks the current number of jobs being processed concurrently.
/// Useful for monitoring worker saturation.
pub fn set_in_flight_jobs(stream: &str, count: f64) {
    gauge!(
        names::IN_FLIGHT_JOBS,
        "stream" => stream.to_string()
    )
    .set(count);
}

/// Helper struct for timing operations and recording metrics.
pub struct MetricsTimer {
    stream: String,
    operation: String,
    start: std::time::Instant,
}

impl MetricsTimer {
    /// Start a new timer for the given stream and operation.
    pub fn new(stream: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            stream: stream.into(),
            operation: operation.into(),
            start: std::time::Instant::now(),
        }
    }

    /// Stop the timer and record the duration.
    pub fn stop(self) {
        let duration = self.start.elapsed();
        record_job_duration(&self.stream, &self.operation, duration);
    }

    /// Get elapsed time without stopping.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        // Auto-record on drop if not manually stopped
        let duration = self.start.elapsed();
        record_job_duration(&self.stream, &self.operation, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_as_str() {
        assert_eq!(JobStatus::Success.as_str(), "success");
        assert_eq!(JobStatus::Failed.as_str(), "failed");
        assert_eq!(JobStatus::Dlq.as_str(), "dlq");
        assert_eq!(JobStatus::Skipped.as_str(), "skipped");
    }

    #[test]
    fn test_error_type_as_str() {
        assert_eq!(ErrorType::Transient.as_str(), "transient");
        assert_eq!(ErrorType::Permanent.as_str(), "permanent");
        assert_eq!(ErrorType::RateLimited.as_str(), "rate_limited");
        assert_eq!(ErrorType::Validation.as_str(), "validation");
        assert_eq!(ErrorType::Connection.as_str(), "connection");
        assert_eq!(ErrorType::Timeout.as_str(), "timeout");
        assert_eq!(ErrorType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_metrics_timer() {
        let timer = MetricsTimer::new("test_stream", "test_op");
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(timer.elapsed().as_millis() >= 10);
    }
}
