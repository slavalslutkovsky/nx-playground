//! Stream worker main processing loop
//!
//! The `StreamWorker` is the main component that ties everything together.

use crate::config::WorkerConfig;
use crate::consumer::StreamConsumer;
use crate::dlq::DlqManager;
use crate::error::{ErrorCategory, StreamError};
use crate::metrics::StreamMetrics;
use crate::registry::{StreamJob, StreamProcessor};
use crate::resilience::{CircuitBreaker, CircuitState, RateLimiter, Resilience};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

/// Stream worker that processes jobs from a Redis stream
pub struct StreamWorker<J: StreamJob, P: StreamProcessor<J>> {
    consumer: StreamConsumer,
    dlq: DlqManager,
    processor: Arc<P>,
    config: WorkerConfig,
    metrics: StreamMetrics,
    resilience: Resilience,
    _marker: std::marker::PhantomData<J>,
}

impl<J: StreamJob, P: StreamProcessor<J>> StreamWorker<J, P> {
    /// Create a new StreamWorker
    pub fn new(redis: ConnectionManager, processor: P, config: WorkerConfig) -> Self {
        let redis = Arc::new(redis);
        let processor_name = processor.name();

        let consumer = StreamConsumer::new(redis.clone(), config.clone());
        let dlq = DlqManager::new(redis.clone(), config.dlq_stream.clone());
        let metrics = StreamMetrics::new(&config.stream_name, processor_name);

        // Set up resilience based on config
        let resilience = if config.enable_circuit_breaker && config.enable_rate_limiter {
            Resilience::with_both(
                CircuitBreaker::default_settings(),
                RateLimiter::new(100, config.rate_limit_rps),
            )
        } else if config.enable_circuit_breaker {
            Resilience::with_circuit_breaker(CircuitBreaker::default_settings())
        } else if config.enable_rate_limiter {
            Resilience::with_rate_limiter(RateLimiter::new(100, config.rate_limit_rps))
        } else {
            Resilience::none()
        };

        Self {
            consumer,
            dlq,
            processor: Arc::new(processor),
            config,
            metrics,
            resilience,
            _marker: std::marker::PhantomData,
        }
    }

    /// Run the worker loop
    ///
    /// The worker will:
    /// 1. Initialize the consumer group
    /// 2. Process pending messages (redeliveries)
    /// 3. Process new messages
    /// 4. Claim abandoned messages periodically
    /// 5. Handle shutdown gracefully
    pub async fn run(&self, mut shutdown_rx: watch::Receiver<bool>) -> Result<(), StreamError> {
        info!(
            stream = %self.config.stream_name,
            consumer_group = %self.config.consumer_group,
            consumer_id = %self.config.consumer_id,
            "Starting stream worker"
        );

        // Initialize consumer group
        self.consumer.init_consumer_group().await?;

        let mut claim_interval = tokio::time::interval(Duration::from_secs(30));
        let mut metrics_interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            tokio::select! {
                // Check for shutdown
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutdown signal received, stopping worker");
                        break;
                    }
                }

                // Claim abandoned messages periodically
                _ = claim_interval.tick() => {
                    if let Err(e) = self.claim_abandoned().await {
                        warn!(error = %e, "Failed to claim abandoned messages");
                    }
                }

                // Update metrics periodically
                _ = metrics_interval.tick() => {
                    if let Err(e) = self.update_metrics().await {
                        debug!(error = %e, "Failed to update metrics");
                    }
                }

                // Main processing loop
                _ = self.process_batch() => {
                    // Continue to next iteration
                }
            }
        }

        info!("Stream worker stopped");
        Ok(())
    }

    /// Process a batch of messages
    async fn process_batch(&self) -> Result<(), StreamError> {
        // Check circuit breaker
        if let Some(cb) = &self.resilience.circuit_breaker {
            if cb.state().await == CircuitState::Open {
                debug!("Circuit breaker open, waiting...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                return Ok(());
            }
        }

        // First, process pending messages (redeliveries)
        let pending: Vec<crate::event::StreamEvent<J>> =
            self.consumer.read_pending(self.config.batch_size).await?;

        let pending_count = pending.len();

        for event in pending {
            if event.is_redelivery() {
                debug!(
                    stream_id = %event.stream_id,
                    job_id = %event.job_id(),
                    delivery_count = event.delivery_count,
                    "Processing redelivered message"
                );
            }
            self.process_event(event).await?;
        }

        // Then read new messages
        let new_events: Vec<crate::event::StreamEvent<J>> =
            self.consumer.read_new(self.config.batch_size).await?;

        for event in new_events {
            self.metrics.job_received();
            self.process_event(event).await?;
        }

        // If no messages, sleep a bit before next poll
        if pending_count == 0 {
            tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
        }

        Ok(())
    }

    /// Process a single event
    async fn process_event(
        &self,
        event: crate::event::StreamEvent<J>,
    ) -> Result<(), StreamError> {
        let stream_id = event.stream_id.clone();
        let job_id = event.job_id();
        let retry_count = event.retry_count();

        debug!(
            stream_id = %stream_id,
            job_id = %job_id,
            retry_count = retry_count,
            "Processing job"
        );

        // Check rate limiter
        if let Some(rl) = &self.resilience.rate_limiter {
            if !rl.try_acquire().await {
                self.metrics.rate_limited();
                tokio::time::sleep(Duration::from_millis(100)).await;
                return Ok(()); // Don't ack - will be redelivered
            }
        }

        let start = Instant::now();
        let result = self.processor.process(&event.job).await;
        let duration = start.elapsed();

        match result {
            Ok(()) => {
                // Success - acknowledge and record metrics
                self.consumer.ack(&stream_id).await?;
                self.metrics.job_processed(duration);
                self.resilience.record_success().await;

                debug!(
                    stream_id = %stream_id,
                    job_id = %job_id,
                    duration_ms = duration.as_millis(),
                    "Job processed successfully"
                );
            }
            Err(e) => {
                self.resilience.record_failure().await;
                self.handle_error(event, e).await?;
            }
        }

        Ok(())
    }

    /// Handle a processing error
    async fn handle_error(
        &self,
        event: crate::event::StreamEvent<J>,
        error: StreamError,
    ) -> Result<(), StreamError> {
        let job_id = event.job_id();
        let retry_count = event.retry_count();
        let category = error.category();

        self.metrics.job_failed(&format!("{:?}", category));

        match category {
            ErrorCategory::Permanent => {
                // Move directly to DLQ
                warn!(
                    job_id = %job_id,
                    error = %error,
                    "Permanent error, moving to DLQ"
                );
                self.move_to_dlq(&event, &error).await?;
            }
            ErrorCategory::Transient | ErrorCategory::RateLimited => {
                if error.should_retry(retry_count) {
                    // Retry with backoff
                    let delay_ms = error.backoff_delay_ms(retry_count);
                    warn!(
                        job_id = %job_id,
                        error = %error,
                        retry_count = retry_count,
                        next_delay_ms = delay_ms,
                        "Transient error, will retry"
                    );

                    self.metrics.job_retried();

                    // Re-add to stream with incremented retry count
                    let retried_job = event.job.with_retry();
                    self.requeue_job(&retried_job).await?;

                    // Acknowledge original message
                    self.consumer.ack(&event.stream_id).await?;

                    // Wait before next processing
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                } else {
                    // Max retries exceeded, move to DLQ
                    error!(
                        job_id = %job_id,
                        error = %error,
                        retry_count = retry_count,
                        "Max retries exceeded, moving to DLQ"
                    );
                    self.move_to_dlq(&event, &error).await?;
                }
            }
        }

        Ok(())
    }

    /// Move a job to the dead letter queue
    async fn move_to_dlq(
        &self,
        event: &crate::event::StreamEvent<J>,
        error: &StreamError,
    ) -> Result<(), StreamError> {
        self.dlq
            .move_to_dlq(&event.job, &error.to_string(), &event.stream_id)
            .await?;

        self.metrics.job_moved_to_dlq();

        // Acknowledge the original message
        self.consumer.ack(&event.stream_id).await?;

        Ok(())
    }

    /// Re-add a job to the stream for retry
    async fn requeue_job(&self, job: &J) -> Result<String, StreamError> {
        use crate::producer::StreamProducer;

        let producer = StreamProducer::from_arc(
            self.consumer.redis(),
            self.config.stream_name.clone(),
        )
        .with_max_length(self.config.max_length);

        producer.send(job).await
    }

    /// Claim abandoned messages from other consumers
    async fn claim_abandoned(&self) -> Result<(), StreamError> {
        let claimed: Vec<crate::event::StreamEvent<J>> = self
            .consumer
            .claim_abandoned(self.config.batch_size)
            .await?;

        for event in claimed {
            self.metrics.message_claimed();
            self.process_event(event).await?;
        }

        Ok(())
    }

    /// Update metrics gauges
    async fn update_metrics(&self) -> Result<(), StreamError> {
        if let Ok(info) = self.consumer.stream_info().await {
            self.metrics.stream_depth(info.length);
            self.metrics.pending_count(info.pending_count);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{StreamDef, StreamJob, StreamProcessor};
    use async_trait::async_trait;
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

    struct TestProcessor;

    #[async_trait]
    impl StreamProcessor<TestJob> for TestProcessor {
        async fn process(&self, _job: &TestJob) -> Result<(), StreamError> {
            Ok(())
        }
        fn name(&self) -> &'static str {
            "test_processor"
        }
    }

    #[test]
    fn test_worker_config_from_stream_def() {
        let config = WorkerConfig::from_stream_def::<TestStream>();
        assert_eq!(config.stream_name, "test:jobs");
        assert_eq!(config.consumer_group, "test_workers");
        assert_eq!(config.dlq_stream, "test:dlq");
    }
}
