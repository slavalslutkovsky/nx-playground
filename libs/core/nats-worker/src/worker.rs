//! NATS JetStream worker for processing jobs.

use crate::config::WorkerConfig;
use crate::consumer::{NatsConsumer, NatsMessage};
use crate::dlq::DlqManager;
use crate::error::NatsError;
use crate::metrics::NatsMetrics;
use async_nats::jetstream::Context;
use messaging::{ErrorCategory, Job, ProcessingError, Processor};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

/// NATS JetStream worker for processing jobs.
pub struct NatsWorker<J: Job, P: Processor<J>> {
    consumer: NatsConsumer,
    dlq: DlqManager,
    processor: Arc<P>,
    config: WorkerConfig,
    metrics: NatsMetrics,
    _marker: std::marker::PhantomData<J>,
}

impl<J: Job, P: Processor<J>> NatsWorker<J, P> {
    /// Create a new NATS worker.
    pub async fn new(
        jetstream: Context,
        processor: P,
        config: WorkerConfig,
    ) -> Result<Self, NatsError> {
        let jetstream = Arc::new(jetstream);
        let processor_name = processor.name();

        let consumer = NatsConsumer::new(jetstream.clone(), config.clone());
        let dlq = DlqManager::new(jetstream.clone(), &config.dlq_stream);
        let metrics = NatsMetrics::new(&config.stream_name, processor_name);

        // Initialize stream and consumer
        consumer.init().await?;

        // Initialize DLQ stream
        dlq.ensure_stream().await?;

        Ok(Self {
            consumer,
            dlq,
            processor: Arc::new(processor),
            config,
            metrics,
            _marker: std::marker::PhantomData,
        })
    }

    /// Run the worker loop.
    ///
    /// The worker will:
    /// 1. Fetch messages in batches
    /// 2. Process each message
    /// 3. Ack on success, nak on transient failure, term on permanent failure
    /// 4. Move permanently failed messages to DLQ
    /// 5. Handle shutdown gracefully
    pub async fn run(&self, mut shutdown_rx: watch::Receiver<bool>) -> Result<(), NatsError> {
        info!(
            stream = %self.config.stream_name,
            consumer = %self.config.consumer_name,
            durable = %self.config.durable_name,
            "Starting NATS worker"
        );

        loop {
            tokio::select! {
                // Check for shutdown
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutdown signal received, stopping worker");
                        break;
                    }
                }

                // Main processing loop
                result = self.process_batch() => {
                    if let Err(e) = result {
                        error!(error = %e, "Error processing batch");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }

        info!("NATS worker stopped");
        Ok(())
    }

    /// Process a batch of messages.
    async fn process_batch(&self) -> Result<(), NatsError> {
        let messages: Vec<NatsMessage<J>> = self.consumer.fetch(self.config.batch_size).await?;

        if messages.is_empty() {
            // No messages, wait before next poll
            tokio::time::sleep(Duration::from_millis(100)).await;
            return Ok(());
        }

        for message in messages {
            self.metrics.job_received();

            if message.is_redelivery() {
                debug!(
                    job_id = %message.job_id(),
                    sequence = message.sequence,
                    delivery_count = message.delivery_count,
                    "Processing redelivered message"
                );
            }

            self.process_message(message).await?;
        }

        Ok(())
    }

    /// Process a single message.
    async fn process_message(&self, message: NatsMessage<J>) -> Result<(), NatsError> {
        let job_id = message.job_id();
        let sequence = message.sequence;
        let retry_count = message.job.retry_count();

        debug!(
            job_id = %job_id,
            sequence = sequence,
            retry_count = retry_count,
            "Processing job"
        );

        let start = Instant::now();
        let result = self.processor.process(&message.job).await;
        let duration = start.elapsed();

        match result {
            Ok(()) => {
                // Success - acknowledge
                message.ack().await?;
                self.metrics.job_processed(duration);

                debug!(
                    job_id = %job_id,
                    sequence = sequence,
                    duration_ms = duration.as_millis(),
                    "Job processed successfully"
                );
            }
            Err(e) => {
                self.handle_error(message, e).await?;
            }
        }

        Ok(())
    }

    /// Handle a processing error.
    async fn handle_error(
        &self,
        message: NatsMessage<J>,
        error: ProcessingError,
    ) -> Result<(), NatsError> {
        let job_id = message.job_id();
        let retry_count = message.job.retry_count();
        let category = error.category();

        self.metrics.job_failed(&format!("{:?}", category));

        match category {
            ErrorCategory::Permanent => {
                // Move to DLQ immediately
                warn!(
                    job_id = %job_id,
                    error = %error,
                    "Permanent error, moving to DLQ"
                );

                self.dlq
                    .move_to_dlq(&message.job, &error.to_string(), message.sequence)
                    .await?;

                self.metrics.job_moved_to_dlq();

                // Terminate (don't redeliver)
                message.term().await?;
            }
            ErrorCategory::Transient | ErrorCategory::RateLimited => {
                if error.should_retry(retry_count) {
                    // Request redelivery with backoff
                    let delay_ms = error.backoff_delay_ms(retry_count);

                    warn!(
                        job_id = %job_id,
                        error = %error,
                        retry_count = retry_count,
                        delay_ms = delay_ms,
                        "Transient error, will retry"
                    );

                    self.metrics.job_retried();

                    // Nak with delay
                    message
                        .nak_with_delay(Duration::from_millis(delay_ms))
                        .await?;
                } else {
                    // Max retries exceeded, move to DLQ
                    error!(
                        job_id = %job_id,
                        error = %error,
                        retry_count = retry_count,
                        "Max retries exceeded, moving to DLQ"
                    );

                    self.dlq
                        .move_to_dlq(&message.job, &error.to_string(), message.sequence)
                        .await?;

                    self.metrics.job_moved_to_dlq();

                    // Terminate
                    message.term().await?;
                }
            }
        }

        Ok(())
    }

    /// Get stream info.
    pub async fn stream_info(&self) -> Result<crate::consumer::StreamInfo, NatsError> {
        self.consumer.stream_info().await
    }

    /// Get DLQ info.
    pub async fn dlq_info(&self) -> Result<crate::consumer::StreamInfo, NatsError> {
        self.dlq.stream_info().await
    }
}
