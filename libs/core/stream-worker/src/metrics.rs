//! Prometheus metrics for stream workers
//!
//! Provides observability into worker performance and health.

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use std::time::Duration;
use tracing::info;

static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// Initialize Prometheus metrics
///
/// Call this once at startup. Subsequent calls are no-ops.
pub fn init_metrics() {
    let _ = PROMETHEUS_HANDLE.get_or_init(|| {
        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install Prometheus recorder");
        info!("Prometheus metrics initialized");
        handle
    });
}

/// Get the Prometheus handle for rendering metrics
pub fn prometheus_handle() -> Option<&'static PrometheusHandle> {
    PROMETHEUS_HANDLE.get()
}

/// Render metrics in Prometheus format
pub fn render_metrics() -> String {
    prometheus_handle()
        .map(|h| h.render())
        .unwrap_or_default()
}

/// Stream worker metrics helper
#[derive(Clone)]
pub struct StreamMetrics {
    /// Stream name for labeling
    stream_name: String,
    /// Processor name for labeling
    processor_name: String,
}

impl StreamMetrics {
    /// Create new StreamMetrics
    pub fn new(stream_name: impl Into<String>, processor_name: impl Into<String>) -> Self {
        Self {
            stream_name: stream_name.into(),
            processor_name: processor_name.into(),
        }
    }

    /// Record a job being received
    pub fn job_received(&self) {
        counter!(
            "stream_worker_jobs_received_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Record a job being processed successfully
    pub fn job_processed(&self, duration: Duration) {
        counter!(
            "stream_worker_jobs_processed_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone(),
            "status" => "success"
        )
        .increment(1);

        histogram!(
            "stream_worker_job_duration_seconds",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .record(duration.as_secs_f64());
    }

    /// Record a job failing
    pub fn job_failed(&self, category: &str) {
        counter!(
            "stream_worker_jobs_processed_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone(),
            "status" => "failed"
        )
        .increment(1);

        counter!(
            "stream_worker_job_errors_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone(),
            "category" => category.to_string()
        )
        .increment(1);
    }

    /// Record a job being retried
    pub fn job_retried(&self) {
        counter!(
            "stream_worker_jobs_retried_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Record a job moved to DLQ
    pub fn job_moved_to_dlq(&self) {
        counter!(
            "stream_worker_jobs_dlq_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Update stream depth gauge
    pub fn stream_depth(&self, depth: i64) {
        gauge!(
            "stream_worker_stream_depth",
            "stream" => self.stream_name.clone()
        )
        .set(depth as f64);
    }

    /// Update pending count gauge
    pub fn pending_count(&self, count: i64) {
        gauge!(
            "stream_worker_pending_count",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .set(count as f64);
    }

    /// Record circuit breaker state change
    pub fn circuit_breaker_state(&self, state: &str) {
        // Use a counter for state transitions
        counter!(
            "stream_worker_circuit_breaker_transitions_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone(),
            "state" => state.to_string()
        )
        .increment(1);
    }

    /// Record rate limiting
    pub fn rate_limited(&self) {
        counter!(
            "stream_worker_rate_limited_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Record message being claimed from abandoned
    pub fn message_claimed(&self) {
        counter!(
            "stream_worker_messages_claimed_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = StreamMetrics::new("test:stream", "test_processor");
        assert_eq!(metrics.stream_name, "test:stream");
        assert_eq!(metrics.processor_name, "test_processor");
    }
}
