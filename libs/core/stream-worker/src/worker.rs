//! Core worker traits and the generic StreamWorker implementation.
//!
//! This module provides:
//! - `StreamJob` trait for job payloads
//! - `StreamProcessor` trait for job processors
//! - `StreamWorker` struct for running the worker loop

use crate::config::WorkerConfig;
use crate::consumer::StreamConsumer;
use crate::error::{ErrorCategory, StreamError};
use crate::metrics::{self, ErrorType, JobStatus};
use crate::resilience::ResilienceLayer;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, Semaphore};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

/// Trait for stream job payloads.
///
/// Domain models that represent jobs in a stream should implement this trait.
/// It provides the necessary methods for the worker to track and retry jobs.
///
/// # Example
///
/// ```rust,ignore
/// use stream_worker::StreamJob;
///
/// #[derive(Clone, Serialize, Deserialize)]
/// struct EmailJob {
///     id: Uuid,
///     to_email: String,
///     subject: String,
///     retry_count: u32,
/// }
///
/// impl StreamJob for EmailJob {
///     fn job_id(&self) -> String {
///         self.id.to_string()
///     }
///
///     fn retry_count(&self) -> u32 {
///         self.retry_count
///     }
///
///     fn with_retry(&self) -> Self {
///         Self {
///             retry_count: self.retry_count + 1,
///             ..self.clone()
///         }
///     }
/// }
/// ```
pub trait StreamJob: Serialize + DeserializeOwned + Send + Sync + Clone {
    /// Returns the job ID for logging and tracking.
    fn job_id(&self) -> String;

    /// Returns the current retry count.
    fn retry_count(&self) -> u32;

    /// Creates a new job with an incremented retry count.
    fn with_retry(&self) -> Self;

    /// Maximum retries allowed before moving to DLQ.
    /// Default: 3 retries.
    fn max_retries(&self) -> u32 {
        3
    }

    /// Check if the job has exceeded max retries.
    fn exceeded_max_retries(&self, max_retries: u32) -> bool {
        self.retry_count() >= max_retries
    }
}

/// Trait for job processors.
///
/// Domain handlers implement this trait to process jobs from the stream.
///
/// # Type Parameters
///
/// * `J` - The job type that this processor handles
///
/// # Example
///
/// ```rust,ignore
/// use stream_worker::{StreamProcessor, StreamError};
///
/// struct EmailProcessor {
///     email_provider: Arc<dyn EmailProvider>,
/// }
///
/// #[async_trait]
/// impl StreamProcessor<EmailJob> for EmailProcessor {
///     async fn process(&self, job: &EmailJob) -> Result<(), StreamError> {
///         self.email_provider.send(job.to_email.clone(), job.subject.clone()).await?;
///         Ok(())
///     }
///
///     fn name(&self) -> &'static str {
///         "EmailProcessor"
///     }
/// }
/// ```
#[async_trait]
pub trait StreamProcessor<J: StreamJob>: Send + Sync {
    /// Process a single job.
    ///
    /// Return `Ok(())` for success, `Err` for failure.
    /// Failed jobs will be retried or moved to DLQ based on configuration.
    async fn process(&self, job: &J) -> Result<(), StreamError>;

    /// Get the processor name for logging.
    fn name(&self) -> &'static str;

    /// Health check for the processor.
    ///
    /// Override this to add custom health checks (e.g., checking external services).
    /// Default: always returns Ok(true).
    async fn health_check(&self) -> Result<bool, StreamError> {
        Ok(true)
    }
}

/// Generic stream worker that processes jobs using a processor.
///
/// This struct encapsulates the worker loop with:
/// - Consumer group management
/// - Pending message recovery
/// - Retry logic with exponential backoff
/// - Dead letter queue handling
/// - Graceful shutdown
/// - **Concurrent job processing** (configurable via `max_concurrent_jobs`)
///
/// # Type Parameters
///
/// * `J` - The job type (must implement `StreamJob`)
/// * `P` - The processor type (must implement `StreamProcessor<J>`)
///
/// # Concurrency
///
/// By default, jobs are processed sequentially (`max_concurrent_jobs = 1`).
/// For higher throughput with independent jobs, increase this value:
///
/// ```rust,ignore
/// let config = WorkerConfig::new("email:jobs")
///     .with_max_concurrent_jobs(10);
/// ```
pub struct StreamWorker<J, P>
where
    J: StreamJob,
    P: StreamProcessor<J>,
{
    consumer: StreamConsumer,
    processor: Arc<P>,
    config: WorkerConfig,
    /// Semaphore to limit concurrent job processing
    concurrency_semaphore: Arc<Semaphore>,
    /// Optional resilience layer (circuit breaker + rate limiter)
    resilience: Option<Arc<ResilienceLayer>>,
    _phantom: PhantomData<J>,
}

impl<J, P> StreamWorker<J, P>
where
    J: StreamJob + 'static,
    P: StreamProcessor<J> + 'static,
{
    /// Create a new stream worker.
    pub fn new(redis: ConnectionManager, processor: P, config: WorkerConfig) -> Self {
        let consumer = StreamConsumer::new(redis, config.clone());
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        Self {
            consumer,
            processor: Arc::new(processor),
            concurrency_semaphore,
            resilience: None,
            config,
            _phantom: PhantomData,
        }
    }

    /// Create a new stream worker with an Arc processor.
    pub fn with_arc_processor(
        redis: ConnectionManager,
        processor: Arc<P>,
        config: WorkerConfig,
    ) -> Self {
        let consumer = StreamConsumer::new(redis, config.clone());
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        Self {
            consumer,
            processor,
            concurrency_semaphore,
            resilience: None,
            config,
            _phantom: PhantomData,
        }
    }

    /// Add a resilience layer (circuit breaker + optional rate limiter).
    ///
    /// When configured, the worker will:
    /// - Check the circuit breaker before processing jobs
    /// - Skip jobs if the circuit is open (fail fast)
    /// - Record successes/failures to the circuit breaker
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stream_worker::{StreamWorker, WorkerConfig, CircuitBreakerConfig, ResilienceLayer};
    ///
    /// let resilience = ResilienceLayer::new(CircuitBreakerConfig::default());
    /// let worker = StreamWorker::new(redis, processor, config)
    ///     .with_resilience(resilience);
    /// ```
    pub fn with_resilience(mut self, resilience: ResilienceLayer) -> Self {
        self.resilience = Some(Arc::new(resilience));
        self
    }

    /// Get a reference to the resilience layer, if configured.
    pub fn resilience(&self) -> Option<&ResilienceLayer> {
        self.resilience.as_ref().map(|r| r.as_ref())
    }

    /// Get a reference to the consumer for health checks.
    pub fn consumer(&self) -> &StreamConsumer {
        &self.consumer
    }

    /// Get a clone of the Redis connection manager.
    pub fn redis(&self) -> ConnectionManager {
        self.consumer.redis().clone()
    }

    /// Run the worker loop.
    ///
    /// This continuously reads jobs from the stream and processes them.
    /// Use the shutdown receiver to gracefully stop the worker.
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) -> Result<(), StreamError> {
        info!(
            consumer_id = %self.config.consumer_id,
            stream = %self.config.stream_name,
            group = %self.config.consumer_group,
            processor = %self.processor.name(),
            "Starting stream worker"
        );

        // Ensure consumer group exists
        self.consumer.ensure_consumer_group().await?;

        // On startup, claim all pending messages
        if let Err(e) = self.consumer.claim_all_pending_on_startup().await {
            warn!(error = %e, "Failed to claim pending messages on startup");
        }

        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);
        let claim_interval = Duration::from_secs(self.config.claim_idle_time_secs * 2);
        let mut last_claim = std::time::Instant::now();
        let is_blocking = self.consumer.is_blocking();

        // Track consecutive errors for exponential backoff
        let mut consecutive_errors: u32 = 0;
        const MAX_BACKOFF_SECS: u64 = 30;

        if is_blocking {
            info!(
                block_timeout_ms = ?self.config.block_timeout_ms,
                claim_interval_secs = %(self.config.claim_idle_time_secs * 2),
                max_concurrent_jobs = %self.config.max_concurrent_jobs,
                batch_size = %self.config.batch_size,
                "Worker running in BLOCKING mode (production-ready)"
            );
        } else {
            info!(
                poll_interval_ms = %self.config.poll_interval_ms,
                claim_interval_secs = %(self.config.claim_idle_time_secs * 2),
                max_concurrent_jobs = %self.config.max_concurrent_jobs,
                batch_size = %self.config.batch_size,
                "Worker running in POLLING mode"
            );
        }

        loop {
            // Check for shutdown signal
            if *shutdown.borrow() {
                info!("Received shutdown signal, stopping worker");
                break;
            }

            // Process messages
            match self.process_batch().await {
                Ok(_) => {
                    if consecutive_errors > 0 {
                        info!("Connection recovered after {} errors", consecutive_errors);
                        consecutive_errors = 0;
                    }
                }
                Err(e) => {
                    // BLOCK timeout is normal behavior - no messages arrived within timeout
                    // This is NOT an error, just continue to the next iteration
                    if e.is_block_timeout() {
                        debug!("BLOCK timeout - no messages, continuing...");
                        continue;
                    }

                    consecutive_errors += 1;

                    if e.is_nogroup_error() {
                        warn!("Consumer group missing, recreating...");
                        if let Err(create_err) = self.consumer.ensure_consumer_group().await {
                            error!(error = %create_err, "Failed to recreate consumer group");
                        }
                    } else if e.is_connection_error() {
                        let backoff_secs = std::cmp::min(
                            2u64.pow(consecutive_errors.min(5)),
                            MAX_BACKOFF_SECS,
                        );
                        warn!(
                            error = %e,
                            consecutive_errors = %consecutive_errors,
                            backoff_secs = %backoff_secs,
                            "Redis connection error, backing off"
                        );
                        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    } else if e.is_timeout() {
                        // General timeout (not BLOCK) - transient, short backoff
                        debug!(error = %e, "Operation timeout, retrying...");
                    } else {
                        error!(error = %e, "Error processing batch");
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }

            // Periodically claim abandoned messages
            if last_claim.elapsed() >= claim_interval {
                if let Err(e) = self.consumer.claim_abandoned_messages().await {
                    debug!(error = %e, "Error claiming abandoned messages");
                }
                last_claim = std::time::Instant::now();
            }

            // In blocking mode, Redis BLOCK handles waiting, so skip sleep
            // In polling mode, wait before next poll
            if !is_blocking {
                tokio::select! {
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            info!("Received shutdown signal, stopping worker");
                            break;
                        }
                    }
                    _ = tokio::time::sleep(poll_interval) => {}
                }
            }
        }

        info!("Stream worker stopped");
        Ok(())
    }

    /// Process a batch of messages.
    ///
    /// If `max_concurrent_jobs > 1`, jobs are processed concurrently using a semaphore
    /// to limit the number of concurrent tasks. Otherwise, jobs are processed sequentially.
    async fn process_batch(&self) -> Result<(), StreamError> {
        let batch_timer = metrics::MetricsTimer::new(&self.config.stream_name, "batch");

        // Collect all jobs (pending + new)
        let pending = self.consumer.read_pending_messages::<J>().await?;
        let new_messages = self.consumer.read_new_messages::<J>().await?;

        let pending_count = pending.len();
        let new_count = new_messages.len();
        let total_jobs = pending_count + new_count;

        if total_jobs == 0 {
            return Ok(());
        }

        // Combine all jobs
        let all_jobs: Vec<(String, J)> = pending.into_iter().chain(new_messages).collect();

        // Process jobs based on concurrency setting
        if self.config.max_concurrent_jobs == 1 {
            // Sequential processing (original behavior)
            for (message_id, job) in all_jobs {
                self.process_job(&message_id, &job).await;
            }
        } else {
            // Concurrent processing with semaphore
            self.process_jobs_concurrent(all_jobs).await;
        }

        // Record batch size metric
        metrics::record_batch_size(&self.config.stream_name, total_jobs);

        // Timer records on drop
        drop(batch_timer);

        Ok(())
    }

    /// Process jobs concurrently using a semaphore to limit parallelism.
    async fn process_jobs_concurrent(&self, jobs: Vec<(String, J)>) {
        let mut join_set: JoinSet<()> = JoinSet::new();

        // Track in-flight jobs for metrics
        let in_flight = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for (message_id, job) in jobs {
            // Clone references for the spawned task
            let semaphore = Arc::clone(&self.concurrency_semaphore);
            let processor = Arc::clone(&self.processor);
            let consumer = self.consumer.redis().clone();
            let config = self.config.clone();
            let in_flight = Arc::clone(&in_flight);
            let stream_name = self.config.stream_name.clone();
            let resilience = self.resilience.clone();

            join_set.spawn(async move {
                // Acquire semaphore permit (blocks if at max concurrency)
                let _permit = semaphore.acquire().await.expect("Semaphore closed");

                // Update in-flight counter
                let current = in_flight.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                metrics::set_in_flight_jobs(&stream_name, current as f64);

                // Process the job
                Self::process_job_static(
                    &processor,
                    consumer,
                    &config,
                    &message_id,
                    &job,
                    resilience.as_ref(),
                )
                .await;

                // Decrement in-flight counter
                let current = in_flight.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) - 1;
                metrics::set_in_flight_jobs(&stream_name, current as f64);
            });
        }

        // Wait for all jobs to complete
        while join_set.join_next().await.is_some() {}
    }

    /// Static method to process a job (used for concurrent processing).
    async fn process_job_static(
        processor: &Arc<P>,
        redis: ConnectionManager,
        config: &WorkerConfig,
        message_id: &str,
        job: &J,
        resilience: Option<&Arc<ResilienceLayer>>,
    ) {
        debug!(
            message_id = %message_id,
            job_id = %job.job_id(),
            "Processing job (concurrent)"
        );

        // Check circuit breaker before processing
        if let Some(layer) = resilience {
            if let Err(e) = layer.check() {
                warn!(
                    message_id = %message_id,
                    job_id = %job.job_id(),
                    error = %e,
                    circuit_state = ?layer.circuit_state(),
                    "Skipping job due to resilience check failure (concurrent)"
                );

                // Re-queue the job for later processing
                Self::requeue_job_static(&redis, config, job)
                    .await
                    .unwrap_or_else(|err| {
                        error!(error = %err, "Failed to requeue job after circuit breaker rejection");
                    });
                Self::ack_message_static(&redis, config, message_id)
                    .await
                    .unwrap_or_else(|err| {
                        error!(error = %err, "Failed to ACK message after circuit breaker rejection");
                    });

                metrics::record_job_processed(&config.stream_name, JobStatus::Skipped);
                return;
            }
        }

        let start = std::time::Instant::now();

        match processor.process(job).await {
            Ok(()) => {
                // Record success to circuit breaker
                if let Some(layer) = resilience {
                    layer.record_success();
                }

                // Record success metric
                metrics::record_job_processed(&config.stream_name, JobStatus::Success);
                metrics::record_job_duration(&config.stream_name, "process", start.elapsed());

                // Acknowledge the message
                let mut conn = redis.clone();
                if let Err(e) = redis::cmd("XACK")
                    .arg(&config.stream_name)
                    .arg(&config.consumer_group)
                    .arg(message_id)
                    .query_async::<()>(&mut conn)
                    .await
                {
                    error!(message_id = %message_id, error = %e, "Failed to ACK message");
                }
            }
            Err(e) => {
                // Record failure to circuit breaker
                if let Some(layer) = resilience {
                    layer.record_failure();
                }
                // Categorize the error for smart retry logic
                let category = e.category();
                let error_type = match category {
                    ErrorCategory::Transient => ErrorType::Transient,
                    ErrorCategory::Permanent => ErrorType::Permanent,
                    ErrorCategory::RateLimited => ErrorType::RateLimited,
                };

                // Record failure metric with error type
                metrics::record_job_processed(&config.stream_name, JobStatus::Failed);
                metrics::record_error(&config.stream_name, error_type);

                warn!(
                    message_id = %message_id,
                    job_id = %job.job_id(),
                    error = %e,
                    error_category = ?category,
                    "Job processing failed (concurrent)"
                );

                // Handle error (retry or DLQ)
                if let Err(handler_err) =
                    Self::handle_job_error_static(&redis, config, job, message_id, e).await
                {
                    error!(
                        message_id = %message_id,
                        error = %handler_err,
                        "Failed to handle job error"
                    );
                    // Still ACK to prevent infinite loop
                    let mut conn = redis.clone();
                    let _ = redis::cmd("XACK")
                        .arg(&config.stream_name)
                        .arg(&config.consumer_group)
                        .arg(message_id)
                        .query_async::<()>(&mut conn)
                        .await;
                }
            }
        }
    }

    /// Static method to handle job errors (used for concurrent processing).
    async fn handle_job_error_static(
        redis: &ConnectionManager,
        config: &WorkerConfig,
        job: &J,
        message_id: &str,
        error: StreamError,
    ) -> Result<(), StreamError> {
        let category = error.category();
        let retry_strategy = category.retry_strategy();

        // Get the effective max retries
        let max_retries = retry_strategy.max_retries().max(config.max_retries);

        // Check if this is a permanent error
        if !category.should_retry() {
            warn!(
                job_id = %job.job_id(),
                error_category = ?category,
                "Permanent error - moving to DLQ without retry (concurrent)"
            );

            metrics::record_job_processed(&config.stream_name, JobStatus::Dlq);

            if config.enable_dlq {
                Self::move_to_dlq_static(redis, config, job, &error.to_string()).await?;
            }

            Self::ack_message_static(redis, config, message_id).await?;
            return Ok(());
        }

        // Check if we've exceeded max retries
        if job.exceeded_max_retries(max_retries) {
            warn!(
                job_id = %job.job_id(),
                max_retries = %max_retries,
                "Job exceeded max retries, moving to DLQ (concurrent)"
            );

            metrics::record_job_processed(&config.stream_name, JobStatus::Dlq);

            if config.enable_dlq {
                Self::move_to_dlq_static(redis, config, job, &error.to_string()).await?;
            }

            Self::ack_message_static(redis, config, message_id).await?;
            return Ok(());
        }

        // Re-queue with incremented retry count
        let next_retry = job.retry_count() + 1;
        metrics::record_retry(&config.stream_name, next_retry);

        let retry_job = job.with_retry();
        Self::requeue_job_static(redis, config, &retry_job).await?;
        Self::ack_message_static(redis, config, message_id).await?;

        Ok(())
    }

    /// Static helper to acknowledge a message.
    async fn ack_message_static(
        redis: &ConnectionManager,
        config: &WorkerConfig,
        message_id: &str,
    ) -> Result<(), StreamError> {
        let mut conn = redis.clone();
        redis::cmd("XACK")
            .arg(&config.stream_name)
            .arg(&config.consumer_group)
            .arg(message_id)
            .query_async::<()>(&mut conn)
            .await
            .map_err(StreamError::Redis)
    }

    /// Static helper to requeue a job.
    async fn requeue_job_static(
        redis: &ConnectionManager,
        config: &WorkerConfig,
        job: &J,
    ) -> Result<(), StreamError> {
        let mut conn = redis.clone();
        let job_json = serde_json::to_string(job)?;

        redis::cmd("XADD")
            .arg(&config.stream_name)
            .arg("*")
            .arg("job")
            .arg(&job_json)
            .query_async::<String>(&mut conn)
            .await
            .map_err(StreamError::Redis)?;

        Ok(())
    }

    /// Static helper to move a job to the DLQ.
    async fn move_to_dlq_static(
        redis: &ConnectionManager,
        config: &WorkerConfig,
        job: &J,
        error: &str,
    ) -> Result<(), StreamError> {
        let mut conn = redis.clone();
        let job_json = serde_json::to_string(job)?;

        redis::cmd("XADD")
            .arg(&config.dlq_stream_name)
            .arg("*")
            .arg("job")
            .arg(&job_json)
            .arg("error")
            .arg(error)
            .query_async::<String>(&mut conn)
            .await
            .map_err(StreamError::Redis)?;

        Ok(())
    }

    /// Process a single job.
    async fn process_job(&self, message_id: &str, job: &J) {
        debug!(
            message_id = %message_id,
            job_id = %job.job_id(),
            "Processing job"
        );

        // Check circuit breaker before processing
        if let Some(ref resilience) = self.resilience {
            if let Err(e) = resilience.check() {
                warn!(
                    message_id = %message_id,
                    job_id = %job.job_id(),
                    error = %e,
                    circuit_state = ?resilience.circuit_state(),
                    "Skipping job due to resilience check failure"
                );

                // Re-queue the job for later processing
                if let Err(requeue_err) = self.consumer.requeue_job(job).await {
                    error!(error = %requeue_err, "Failed to requeue job after circuit breaker rejection");
                }
                if let Err(ack_err) = self.consumer.ack_message(message_id).await {
                    error!(error = %ack_err, "Failed to ACK message after circuit breaker rejection");
                }

                metrics::record_job_processed(&self.config.stream_name, JobStatus::Skipped);
                return;
            }
        }

        let job_timer = metrics::MetricsTimer::new(&self.config.stream_name, "process");
        let start = std::time::Instant::now();

        match self.processor.process(job).await {
            Ok(()) => {
                // Record success to circuit breaker
                if let Some(ref resilience) = self.resilience {
                    resilience.record_success();
                }

                // Record success metric
                metrics::record_job_processed(&self.config.stream_name, JobStatus::Success);
                metrics::record_job_duration(
                    &self.config.stream_name,
                    "process",
                    start.elapsed(),
                );

                // Acknowledge the message
                if let Err(e) = self.consumer.ack_message(message_id).await {
                    error!(message_id = %message_id, error = %e, "Failed to ACK message");
                }
            }
            Err(e) => {
                // Record failure to circuit breaker
                if let Some(ref resilience) = self.resilience {
                    resilience.record_failure();
                }
                // Categorize the error for smart retry logic
                let category = e.category();
                let error_type = match category {
                    ErrorCategory::Transient => ErrorType::Transient,
                    ErrorCategory::Permanent => ErrorType::Permanent,
                    ErrorCategory::RateLimited => ErrorType::RateLimited,
                };

                // Record failure metric with error type
                metrics::record_job_processed(&self.config.stream_name, JobStatus::Failed);
                metrics::record_error(&self.config.stream_name, error_type);

                warn!(
                    message_id = %message_id,
                    job_id = %job.job_id(),
                    error = %e,
                    error_category = ?category,
                    "Job processing failed"
                );

                if let Err(handler_err) = self.handle_job_error(job, message_id, e).await {
                    error!(
                        message_id = %message_id,
                        error = %handler_err,
                        "Failed to handle job error"
                    );
                    // Still ACK to prevent infinite loop
                    let _ = self.consumer.ack_message(message_id).await;
                }
            }
        }

        // Prevent double recording (MetricsTimer records on drop)
        std::mem::forget(job_timer);
    }

    /// Handle a job processing error with smart retry logic.
    ///
    /// Uses error categorization to determine retry behavior:
    /// - **Transient**: Retry with exponential backoff (connection issues, timeouts)
    /// - **Permanent**: Move to DLQ immediately (invalid data, auth failures)
    /// - **RateLimited**: Retry with longer backoff (429s, quota exceeded)
    async fn handle_job_error(
        &self,
        job: &J,
        message_id: &str,
        error: StreamError,
    ) -> Result<(), StreamError> {
        let category = error.category();
        let retry_strategy = category.retry_strategy();

        error!(
            job_id = %job.job_id(),
            message_id = %message_id,
            retry_count = %job.retry_count(),
            error = %error,
            error_category = ?category,
            "Failed to process job"
        );

        // Get the effective max retries (use category-specific or config value)
        let max_retries = retry_strategy.max_retries().max(self.config.max_retries);

        // Check if this is a permanent error (no retry regardless of count)
        if !category.should_retry() {
            warn!(
                job_id = %job.job_id(),
                error_category = ?category,
                "Permanent error - moving to DLQ without retry"
            );

            // Record DLQ metric
            metrics::record_job_processed(&self.config.stream_name, JobStatus::Dlq);

            if self.config.enable_dlq {
                self.consumer.move_to_dlq(job, &error.to_string()).await?;
            }

            // Acknowledge the original message
            self.consumer.ack_message(message_id).await?;
            return Ok(());
        }

        // Check if we've exceeded max retries
        if job.exceeded_max_retries(max_retries) {
            warn!(
                job_id = %job.job_id(),
                max_retries = %max_retries,
                error_category = ?category,
                "Job exceeded max retries, moving to DLQ"
            );

            // Record DLQ metric
            metrics::record_job_processed(&self.config.stream_name, JobStatus::Dlq);

            if self.config.enable_dlq {
                self.consumer.move_to_dlq(job, &error.to_string()).await?;
            }

            // Acknowledge the original message
            self.consumer.ack_message(message_id).await?;
            return Ok(());
        }

        // Calculate delay using retry strategy with jitter
        let attempt = job.retry_count();
        let delay = retry_strategy
            .delay_for_attempt(attempt)
            .unwrap_or_else(|| Duration::from_secs(2u64.pow(attempt)));

        // Record retry metric
        let next_retry = attempt + 1;
        metrics::record_retry(&self.config.stream_name, next_retry);

        info!(
            job_id = %job.job_id(),
            retry_attempt = %next_retry,
            delay_ms = %delay.as_millis(),
            error_category = ?category,
            "Scheduling job retry with backoff"
        );

        // Re-queue with incremented retry count
        let retry_job = job.with_retry();
        self.consumer.requeue_job(&retry_job).await?;

        // Acknowledge the original message
        self.consumer.ack_message(message_id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestJob {
        id: String,
        data: String,
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
                retry_count: self.retry_count + 1,
                ..self.clone()
            }
        }
    }

    #[test]
    fn test_stream_job_trait() {
        let job = TestJob {
            id: "job-1".to_string(),
            data: "test".to_string(),
            retry_count: 0,
        };

        assert_eq!(job.job_id(), "job-1");
        assert_eq!(job.retry_count(), 0);
        assert_eq!(job.max_retries(), 3);
        assert!(!job.exceeded_max_retries(3));

        let retry = job.with_retry();
        assert_eq!(retry.retry_count(), 1);
    }
}
