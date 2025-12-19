//! Email worker for processing email jobs from the Redis stream.
//!
//! Note: This module is being deprecated in favor of using the generic
//! `StreamWorker` from `stream-worker` crate with `EmailProcessor`.
//! See `apps/zerg/email-worker` for the new implementation.

use crate::error::{NotificationError, NotificationResult};
use crate::models::EmailJob;
use crate::providers::{EmailContent, EmailProvider};
use crate::templates::TemplateEngine;
use redis::aio::ConnectionManager;
use stream_worker::StreamJob;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for the email worker.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Redis stream name.
    pub stream_name: String,
    /// Consumer group name.
    pub consumer_group: String,
    /// Worker/consumer ID.
    pub consumer_id: String,
    /// Batch size for reading jobs.
    pub batch_size: usize,
    /// Poll interval in milliseconds (how often to check for new messages).
    pub poll_interval_ms: u64,
    /// Maximum retry attempts before moving to DLQ.
    pub max_retries: u32,
    /// Dead letter queue stream name.
    pub dlq_stream_name: String,
    /// Time in seconds before claiming abandoned messages.
    pub claim_idle_time_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            stream_name: std::env::var("EMAIL_STREAM_NAME")
                .unwrap_or_else(|_| "email:jobs".to_string()),
            consumer_group: std::env::var("EMAIL_CONSUMER_GROUP")
                .unwrap_or_else(|_| "email_workers".to_string()),
            consumer_id: format!("worker-{}", Uuid::new_v4()),
            batch_size: 10,
            // Poll interval: 500ms for responsive message processing
            // Can be increased in production if lower latency isn't critical
            poll_interval_ms: std::env::var("EMAIL_POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),
            max_retries: std::env::var("EMAIL_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            dlq_stream_name: std::env::var("EMAIL_DLQ_STREAM_NAME")
                .unwrap_or_else(|_| "email:dlq".to_string()),
            // Claim idle time: 5s to quickly recover messages from crashed workers
            claim_idle_time_secs: 5,
        }
    }
}

/// Email worker that processes jobs from the Redis stream.
pub struct EmailWorker<P: EmailProvider> {
    redis: Arc<ConnectionManager>,
    provider: Arc<P>,
    templates: Arc<TemplateEngine>,
    config: WorkerConfig,
}

impl<P: EmailProvider + 'static> EmailWorker<P> {
    /// Create a new email worker.
    pub fn new(
        redis: ConnectionManager,
        provider: P,
        templates: TemplateEngine,
        config: WorkerConfig,
    ) -> Self {
        Self {
            redis: Arc::new(redis),
            provider: Arc::new(provider),
            templates: Arc::new(templates),
            config,
        }
    }

    /// Create a worker with default config.
    pub fn with_default_config(
        redis: ConnectionManager,
        provider: P,
        templates: TemplateEngine,
    ) -> Self {
        Self::new(redis, provider, templates, WorkerConfig::default())
    }

    /// Run the worker loop.
    ///
    /// This will continuously read jobs from the stream and process them.
    /// Use the shutdown receiver to gracefully stop the worker.
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) -> NotificationResult<()> {
        info!(
            consumer_id = %self.config.consumer_id,
            stream = %self.config.stream_name,
            group = %self.config.consumer_group,
            "Starting email worker"
        );

        // Ensure consumer group exists
        self.ensure_consumer_group().await?;

        // On startup, aggressively claim ALL pending messages from any consumer
        // This is critical when workers restart frequently (bacon/cargo-watch)
        if let Err(e) = self.claim_all_pending_on_startup().await {
            warn!(error = %e, "Failed to claim pending messages on startup");
        }

        // Use configured poll interval
        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);
        let claim_interval = Duration::from_secs(self.config.claim_idle_time_secs * 2);
        let mut last_claim = std::time::Instant::now();

        // Track consecutive errors for exponential backoff
        let mut consecutive_errors: u32 = 0;
        const MAX_BACKOFF_SECS: u64 = 30;

        info!(
            poll_interval_ms = %self.config.poll_interval_ms,
            claim_interval_secs = %(self.config.claim_idle_time_secs * 2),
            "Worker polling configuration"
        );

        loop {
            // Check for shutdown signal
            if *shutdown.borrow() {
                info!("Received shutdown signal, stopping worker");
                break;
            }

            // Process messages (pending + new)
            match self.process_batch().await {
                Ok(_) => {
                    // Reset error counter on success
                    if consecutive_errors > 0 {
                        info!("Connection recovered after {} errors", consecutive_errors);
                        consecutive_errors = 0;
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let err_str = e.to_string();

                    // If consumer group was deleted, recreate it
                    if err_str.contains("NOGROUP") {
                        warn!("Consumer group missing, recreating...");
                        if let Err(create_err) = self.ensure_consumer_group().await {
                            error!(error = %create_err, "Failed to recreate consumer group");
                        }
                    } else if Self::is_connection_error(&err_str) {
                        // Connection error - use exponential backoff
                        let backoff_secs = std::cmp::min(
                            2u64.pow(consecutive_errors.min(5)),
                            MAX_BACKOFF_SECS
                        );
                        warn!(
                            error = %e,
                            consecutive_errors = %consecutive_errors,
                            backoff_secs = %backoff_secs,
                            "Redis connection error, backing off"
                        );
                        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    } else {
                        error!(error = %e, "Error processing batch");
                    }

                    // Always wait at least 1 second on error
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }

            // Periodically claim abandoned messages from crashed workers
            if last_claim.elapsed() >= claim_interval {
                if let Err(e) = self.claim_abandoned_messages().await {
                    // Don't count claim errors as consecutive errors
                    debug!(error = %e, "Error claiming abandoned messages");
                }
                last_claim = std::time::Instant::now();
            }

            // Poll interval - wait before checking for more messages
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

        info!("Email worker stopped");
        Ok(())
    }

    /// Check if an error is a Redis connection error
    fn is_connection_error(err_str: &str) -> bool {
        let lower = err_str.to_lowercase();
        lower.contains("connection")
            || lower.contains("disconnected")
            || lower.contains("broken pipe")
            || lower.contains("reset by peer")
            || lower.contains("refused")
            || lower.contains("timed out")
            || lower.contains("eof")
            || lower.contains("io error")
    }

    /// Ensure the consumer group exists.
    async fn ensure_consumer_group(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                info!("Created consumer group");
                Ok(())
            }
            Err(e) if e.to_string().contains("BUSYGROUP") => {
                debug!("Consumer group already exists");
                Ok(())
            }
            Err(e) => Err(NotificationError::QueueError(e.to_string())),
        }
    }

    /// Process a batch of messages from the stream.
    async fn process_batch(&self) -> NotificationResult<()> {
        // First, process any pending messages (claimed from dead workers or not yet ACKed)
        self.process_pending_messages().await?;

        // Then, read and process new messages
        self.process_new_messages().await?;

        Ok(())
    }

    /// Process pending messages that belong to this consumer (claimed or not yet ACKed).
    async fn process_pending_messages(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        // Read pending messages using "0" instead of ">" to get our own pending entries
        let opts = StreamReadOptions::default()
            .group(&self.config.consumer_group, &self.config.consumer_id)
            .count(self.config.batch_size);

        let result: Result<StreamReadReply, _> = conn
            .xread_options(&[&self.config.stream_name], &["0"], &opts)
            .await;

        let reply = match result {
            Ok(reply) => reply,
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("timeout") || err_str.contains("timed out") {
                    return Ok(());
                }
                return Err(NotificationError::QueueError(e.to_string()));
            }
        };

        for stream_key in reply.keys {
            let count = stream_key.ids.len();
            if count > 0 {
                debug!(count = count, "Processing pending messages");
                for message in stream_key.ids {
                    if let Err(e) = self.process_message(message).await {
                        error!(error = %e, "Error processing pending message");
                    }
                }
            }
        }

        Ok(())
    }

    /// Process new messages from the stream.
    /// Uses non-blocking polling instead of Redis BLOCK to avoid ConnectionManager issues.
    async fn process_new_messages(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        // Use non-blocking read - more reliable with ConnectionManager
        // The main loop handles the polling delay
        let opts = StreamReadOptions::default()
            .group(&self.config.consumer_group, &self.config.consumer_id)
            .count(self.config.batch_size);
        // NOTE: No .block() - we poll instead for better reliability

        let result: Result<StreamReadReply, _> = conn
            .xread_options(&[&self.config.stream_name], &[">"], &opts)
            .await;

        let reply = match result {
            Ok(reply) => reply,
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                // Connection errors should be retried
                if err_str.contains("timeout") || err_str.contains("timed out") {
                    return Ok(());
                }
                return Err(NotificationError::QueueError(e.to_string()));
            }
        };

        for stream_key in reply.keys {
            let count = stream_key.ids.len();
            if count > 0 {
                info!(count = count, "Received new messages");
                for message in stream_key.ids {
                    if let Err(e) = self.process_message(message).await {
                        error!(error = %e, "Error processing new message");
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single message from the stream.
    async fn process_message(&self, message: redis::streams::StreamId) -> NotificationResult<()> {
        let message_id = message.id.clone();
        debug!(message_id = %message_id, "Processing message");

        // Parse the job from the message
        let job_result = self.parse_job(&message.map);

        match job_result {
            Ok(job) => {
                debug!(message_id = %message_id, job_id = %job.id, "Parsed job successfully");
                match self.process_job(&job).await {
                    Ok(_) => {
                        // Acknowledge the message
                        if let Err(e) = self.ack_message(&message_id).await {
                            error!(message_id = %message_id, error = %e, "Failed to ACK message");
                        }
                    }
                    Err(e) => {
                        warn!(message_id = %message_id, error = %e, "Job processing failed");
                        // Try to handle the error
                        if let Err(handler_err) = self.handle_job_error(&job, &message_id, e).await {
                            error!(message_id = %message_id, error = %handler_err, "Failed to handle job error");
                            // Still ACK to prevent infinite loop
                            let _ = self.ack_message(&message_id).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!(message_id = %message_id, error = %e, "Failed to parse job, moving to DLQ");
                // Move unparseable messages to DLQ and ACK
                let _ = self.move_to_dlq_raw(&message_id, &message.map).await;
                let _ = self.ack_message(&message_id).await;
            }
        }

        Ok(())
    }

    /// Parse a job from the Redis stream message.
    fn parse_job(&self, map: &HashMap<String, redis::Value>) -> NotificationResult<EmailJob> {
        let job_value = map
            .get("job")
            .ok_or_else(|| NotificationError::Internal("Missing 'job' field in message".to_string()))?;

        let job_str = match job_value {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            redis::Value::SimpleString(s) => s.clone(),
            _ => {
                return Err(NotificationError::Internal(
                    "Invalid 'job' field type".to_string(),
                ))
            }
        };

        let job: EmailJob = serde_json::from_str(&job_str)?;
        Ok(job)
    }

    /// Process a single email job.
    async fn process_job(&self, job: &EmailJob) -> NotificationResult<()> {
        info!(
            job_id = %job.id,
            email_type = %job.email_type,
            to = %job.to_email,
            retry_count = %job.retry_count,
            "Processing email job"
        );

        // Render the email template
        let rendered = self.templates.render_by_type(&job.email_type, &job.template_vars)?;

        // Create email content
        let email = EmailContent {
            to_email: job.to_email.clone(),
            to_name: job.to_name.clone(),
            subject: job.subject.clone(),
            html_body: rendered.html,
            text_body: rendered.text,
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
        };

        // Send via provider
        let result = self.provider.send(&email).await?;

        info!(
            job_id = %job.id,
            email_type = %job.email_type,
            to = %job.to_email,
            message_id = ?result.message_id,
            "Successfully sent email"
        );

        Ok(())
    }

    /// Handle a job processing error.
    async fn handle_job_error(
        &self,
        job: &EmailJob,
        message_id: &str,
        error: NotificationError,
    ) -> NotificationResult<()> {
        error!(
            job_id = %job.id,
            message_id = %message_id,
            retry_count = %job.retry_count,
            error = %error,
            "Failed to process email job"
        );

        if job.exceeded_max_retries(self.config.max_retries) {
            // Move to dead letter queue
            warn!(
                job_id = %job.id,
                max_retries = %self.config.max_retries,
                "Job exceeded max retries, moving to DLQ"
            );
            self.move_to_dlq(job, &error.to_string()).await?;
        } else {
            // Re-queue with incremented retry count
            let retry_job = job.with_retry();
            self.requeue_job(&retry_job).await?;

            // Add exponential backoff delay
            let delay = Duration::from_secs(2u64.pow(job.retry_count));
            debug!(
                job_id = %job.id,
                delay_secs = %delay.as_secs(),
                "Will retry job after delay"
            );
        }

        // Acknowledge the original message
        self.ack_message(message_id).await?;

        Ok(())
    }

    /// Acknowledge a message.
    async fn ack_message(&self, message_id: &str) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        let _: () = conn
            .xack(
                &self.config.stream_name,
                &self.config.consumer_group,
                &[message_id],
            )
            .await?;

        debug!(message_id = %message_id, "Acknowledged message");
        Ok(())
    }

    /// Re-queue a job for retry.
    async fn requeue_job(&self, job: &EmailJob) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        let _: String = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("*")
            .arg("job")
            .arg(&job_json)
            .query_async(&mut conn)
            .await?;

        debug!(job_id = %job.id, retry_count = %job.retry_count, "Re-queued job for retry");
        Ok(())
    }

    /// Move a job to the dead letter queue.
    async fn move_to_dlq(&self, job: &EmailJob, error: &str) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        let dlq_entry = serde_json::json!({
            "job": job,
            "error": error,
            "failed_at": chrono::Utc::now().to_rfc3339(),
        });

        let _: String = redis::cmd("XADD")
            .arg(&self.config.dlq_stream_name)
            .arg("*")
            .arg("data")
            .arg(dlq_entry.to_string())
            .query_async(&mut conn)
            .await?;

        warn!(job_id = %job.id, "Moved job to DLQ");
        Ok(())
    }

    /// Move a raw message to the DLQ (for unparseable messages).
    async fn move_to_dlq_raw(
        &self,
        message_id: &str,
        map: &HashMap<String, redis::Value>,
    ) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        let raw_data = format!("{:?}", map);
        let dlq_entry = serde_json::json!({
            "raw_message": raw_data,
            "original_id": message_id,
            "error": "Failed to parse job",
            "failed_at": chrono::Utc::now().to_rfc3339(),
        });

        let _: String = redis::cmd("XADD")
            .arg(&self.config.dlq_stream_name)
            .arg("*")
            .arg("data")
            .arg(dlq_entry.to_string())
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    /// Claim ALL pending messages on startup, regardless of idle time.
    /// This is crucial when workers restart frequently (bacon/cargo-watch creates new consumer IDs).
    async fn claim_all_pending_on_startup(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();
        let mut total_claimed = 0;
        let mut start_id = "0-0".to_string();

        loop {
            // XAUTOCLAIM with min-idle-time of 0 to claim ALL pending messages
            let result: redis::Value = redis::cmd("XAUTOCLAIM")
                .arg(&self.config.stream_name)
                .arg(&self.config.consumer_group)
                .arg(&self.config.consumer_id)
                .arg(0) // min-idle-time = 0 to claim everything
                .arg(&start_id)
                .arg("COUNT")
                .arg(100) // Larger batch on startup
                .query_async(&mut conn)
                .await
                .unwrap_or(redis::Value::Nil);

            // Parse the XAUTOCLAIM response: [next-start-id, [[msg-id, fields], ...], [deleted-ids]]
            if let redis::Value::Array(arr) = &result {
                if arr.len() >= 2 {
                    // Get next start ID for pagination
                    if let redis::Value::BulkString(next_id) = &arr[0] {
                        let next = String::from_utf8_lossy(next_id).to_string();
                        if next == "0-0" {
                            // No more messages to claim
                            break;
                        }
                        start_id = next;
                    } else {
                        break;
                    }

                    // Count claimed messages
                    if let redis::Value::Array(messages) = &arr[1] {
                        total_claimed += messages.len();
                        if messages.is_empty() {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if total_claimed > 0 {
            info!(
                count = total_claimed,
                consumer = %self.config.consumer_id,
                "Claimed pending messages on startup"
            );
        }

        Ok(())
    }

    /// Claim abandoned messages from crashed workers using XAUTOCLAIM.
    /// This is more reliable than XPENDING + XCLAIM as it handles both in one command.
    async fn claim_abandoned_messages(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        // Get pending messages that have been idle for too long
        let idle_time_ms = self.config.claim_idle_time_secs * 1000;

        // XAUTOCLAIM stream group consumer min-idle-time start [COUNT count]
        // Returns: [next-start-id, [[message-id, [field, value, ...]], ...], [deleted-ids]]
        let result: redis::Value = redis::cmd("XAUTOCLAIM")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(&self.config.consumer_id)
            .arg(idle_time_ms)
            .arg("0-0") // Start from beginning
            .arg("COUNT")
            .arg(10)
            .query_async(&mut conn)
            .await
            .unwrap_or(redis::Value::Nil);

        // Parse the XAUTOCLAIM response to count claimed messages
        if let redis::Value::Array(arr) = &result {
            if arr.len() >= 2 {
                if let redis::Value::Array(messages) = &arr[1] {
                    if !messages.is_empty() {
                        info!(
                            count = messages.len(),
                            consumer = %self.config.consumer_id,
                            "Claimed abandoned messages"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.stream_name, "email:jobs");
        assert_eq!(config.consumer_group, "email_workers");
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.max_retries, 3);
    }
}
