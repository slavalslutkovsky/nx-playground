//! Prometheus metrics for NATS worker.

use metrics::{counter, gauge, histogram};
use std::time::Duration;

/// Metrics for NATS worker.
#[derive(Clone)]
pub struct NatsMetrics {
    stream_name: String,
    processor_name: String,
}

impl NatsMetrics {
    /// Create new metrics.
    pub fn new(stream_name: &str, processor_name: &str) -> Self {
        Self {
            stream_name: stream_name.to_string(),
            processor_name: processor_name.to_string(),
        }
    }

    /// Record a job received.
    pub fn job_received(&self) {
        counter!(
            "nats_worker_jobs_received_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Record a job processed successfully.
    pub fn job_processed(&self, duration: Duration) {
        counter!(
            "nats_worker_jobs_processed_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);

        histogram!(
            "nats_worker_job_duration_seconds",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .record(duration.as_secs_f64());
    }

    /// Record a job failed.
    pub fn job_failed(&self, error_category: &str) {
        counter!(
            "nats_worker_jobs_failed_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone(),
            "category" => error_category.to_string()
        )
        .increment(1);
    }

    /// Record a job moved to DLQ.
    pub fn job_moved_to_dlq(&self) {
        counter!(
            "nats_worker_jobs_moved_to_dlq_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Record a job retried.
    pub fn job_retried(&self) {
        counter!(
            "nats_worker_jobs_retried_total",
            "stream" => self.stream_name.clone(),
            "processor" => self.processor_name.clone()
        )
        .increment(1);
    }

    /// Update stream depth gauge.
    pub fn stream_depth(&self, depth: u64) {
        gauge!(
            "nats_worker_stream_depth",
            "stream" => self.stream_name.clone()
        )
        .set(depth as f64);
    }

    /// Update DLQ depth gauge.
    pub fn dlq_depth(&self, depth: u64) {
        gauge!(
            "nats_worker_dlq_depth",
            "stream" => self.stream_name.clone()
        )
        .set(depth as f64);
    }
}

/// Initialize Prometheus metrics.
pub fn init_metrics() -> metrics_exporter_prometheus::PrometheusHandle {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}
