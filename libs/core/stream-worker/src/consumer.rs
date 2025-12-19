//! Redis stream consumer implementation.
//!
//! This module provides the `StreamConsumer` struct that handles:
//! - Reading messages from Redis streams
//! - Consumer group management
//! - Message acknowledgment
//! - Claiming abandoned messages
//! - Dead letter queue operations

use crate::config::WorkerConfig;
use crate::error::StreamError;
use crate::worker::StreamJob;
use redis::aio::ConnectionManager;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::AsyncCommands;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Redis stream consumer for reading and processing messages.
///
/// Handles all Redis stream operations including:
/// - Consumer group creation and management
/// - Reading pending and new messages
/// - Acknowledging processed messages
/// - Claiming abandoned messages from crashed workers
/// - Moving failed jobs to dead letter queue
pub struct StreamConsumer {
    redis: Arc<ConnectionManager>,
    config: WorkerConfig,
}

impl StreamConsumer {
    /// Create a new stream consumer.
    pub fn new(redis: ConnectionManager, config: WorkerConfig) -> Self {
        Self {
            redis: Arc::new(redis),
            config,
        }
    }

    /// Get a reference to the Redis connection manager.
    pub fn redis(&self) -> &ConnectionManager {
        &self.redis
    }

    /// Get the stream name.
    pub fn stream_name(&self) -> &str {
        &self.config.stream_name
    }

    /// Ensure the consumer group exists.
    ///
    /// Creates the consumer group if it doesn't exist.
    /// Also creates the stream if it doesn't exist (MKSTREAM).
    pub async fn ensure_consumer_group(&self) -> Result<(), StreamError> {
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
                info!("Created consumer group '{}'", self.config.consumer_group);
                Ok(())
            }
            Err(e) if e.to_string().contains("BUSYGROUP") => {
                debug!("Consumer group '{}' already exists", self.config.consumer_group);
                Ok(())
            }
            Err(e) => Err(StreamError::ConsumerGroup(e.to_string())),
        }
    }

    /// Read pending messages assigned to this consumer.
    ///
    /// These are messages that were delivered but not yet acknowledged.
    pub async fn read_pending_messages<J: StreamJob>(&self) -> Result<Vec<(String, J)>, StreamError> {
        let mut conn = (*self.redis).clone();

        let opts = StreamReadOptions::default()
            .group(&self.config.consumer_group, &self.config.consumer_id)
            .count(self.config.batch_size);

        // Use "0" to read pending messages (not ">")
        let result: Result<StreamReadReply, _> = conn
            .xread_options(&[&self.config.stream_name], &["0"], &opts)
            .await;

        match result {
            Ok(reply) => self.parse_messages(reply),
            Err(e) if e.to_string().to_lowercase().contains("timeout") => Ok(vec![]),
            Err(e) => Err(StreamError::Redis(e)),
        }
    }

    /// Read new messages from the stream.
    ///
    /// Uses either blocking or polling mode depending on configuration:
    /// - Blocking mode: More efficient, waits on Redis for messages
    /// - Polling mode: Less efficient but simpler shutdown handling
    pub async fn read_new_messages<J: StreamJob>(&self) -> Result<Vec<(String, J)>, StreamError> {
        let mut conn = (*self.redis).clone();

        let mut opts = StreamReadOptions::default()
            .group(&self.config.consumer_group, &self.config.consumer_id)
            .count(self.config.batch_size);

        // Use blocking mode if configured
        if let Some(timeout_ms) = self.config.block_timeout_ms {
            opts = opts.block(timeout_ms as usize);
        }

        // Use ">" to read only new messages
        let result: Result<StreamReadReply, _> = conn
            .xread_options(&[&self.config.stream_name], &[">"], &opts)
            .await;

        match result {
            Ok(reply) => {
                let messages = self.parse_messages(reply)?;
                if !messages.is_empty() {
                    info!(count = messages.len(), "Received new messages");
                }
                Ok(messages)
            }
            // BLOCK timeout returns nil/empty - this is normal, not an error
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                // Handle various timeout patterns from Redis/connection layer
                if err_str.contains("timeout")
                    || err_str.contains("timed out")
                    || err_str.contains("nil")
                    || (err_str.contains("response") && err_str.contains("type"))
                {
                    // BLOCK timeout - no messages arrived, return empty
                    Ok(vec![])
                } else {
                    Err(StreamError::Redis(e))
                }
            }
        }
    }

    /// Check if blocking mode is enabled.
    pub fn is_blocking(&self) -> bool {
        self.config.block_timeout_ms.is_some()
    }

    /// Parse messages from a StreamReadReply.
    fn parse_messages<J: StreamJob>(&self, reply: StreamReadReply) -> Result<Vec<(String, J)>, StreamError> {
        let mut jobs = Vec::new();

        for stream_key in reply.keys {
            for message in stream_key.ids {
                match self.parse_job::<J>(&message.map) {
                    Ok(job) => {
                        jobs.push((message.id, job));
                    }
                    Err(e) => {
                        warn!(
                            message_id = %message.id,
                            error = %e,
                            "Failed to parse job, will move to DLQ"
                        );
                        // We'll handle unparseable messages in the caller
                    }
                }
            }
        }

        Ok(jobs)
    }

    /// Parse a job from the Redis stream message.
    fn parse_job<J: StreamJob>(&self, map: &HashMap<String, redis::Value>) -> Result<J, StreamError> {
        let job_value = map
            .get("job")
            .ok_or_else(|| StreamError::JobParsing("Missing 'job' field in message".to_string()))?;

        let job_str = match job_value {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            redis::Value::SimpleString(s) => s.clone(),
            _ => {
                return Err(StreamError::JobParsing(
                    "Invalid 'job' field type".to_string(),
                ))
            }
        };

        let job: J = serde_json::from_str(&job_str)?;
        Ok(job)
    }

    /// Acknowledge a message.
    pub async fn ack_message(&self, message_id: &str) -> Result<(), StreamError> {
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
    pub async fn requeue_job<J: StreamJob>(&self, job: &J) -> Result<(), StreamError> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        let _: String = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("*")
            .arg("job")
            .arg(&job_json)
            .query_async(&mut conn)
            .await?;

        debug!(job_id = %job.job_id(), retry_count = %job.retry_count(), "Re-queued job for retry");
        Ok(())
    }

    /// Move a job to the dead letter queue.
    pub async fn move_to_dlq<J: Serialize>(&self, job: &J, error: &str) -> Result<(), StreamError> {
        if !self.config.enable_dlq {
            return Ok(());
        }

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

        warn!("Moved job to DLQ: {}", self.config.dlq_stream_name);
        Ok(())
    }

    /// Move a raw message to the DLQ (for unparseable messages).
    pub async fn move_raw_to_dlq(
        &self,
        message_id: &str,
        raw_data: &str,
    ) -> Result<(), StreamError> {
        if !self.config.enable_dlq {
            return Ok(());
        }

        let mut conn = (*self.redis).clone();

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

    /// Claim all pending messages on startup.
    ///
    /// This is critical when workers restart frequently, as the new worker
    /// gets a new consumer ID and needs to claim messages from the old ID.
    pub async fn claim_all_pending_on_startup(&self) -> Result<usize, StreamError> {
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
                .arg(100)
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

        Ok(total_claimed)
    }

    /// Claim abandoned messages from crashed workers.
    pub async fn claim_abandoned_messages(&self) -> Result<usize, StreamError> {
        let mut conn = (*self.redis).clone();

        let idle_time_ms = self.config.claim_idle_time_secs * 1000;

        let result: redis::Value = redis::cmd("XAUTOCLAIM")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(&self.config.consumer_id)
            .arg(idle_time_ms)
            .arg("0-0")
            .arg("COUNT")
            .arg(10)
            .query_async(&mut conn)
            .await
            .unwrap_or(redis::Value::Nil);

        let mut claimed = 0;
        if let redis::Value::Array(arr) = &result {
            if arr.len() >= 2 {
                if let redis::Value::Array(messages) = &arr[1] {
                    claimed = messages.len();
                    if claimed > 0 {
                        info!(
                            count = claimed,
                            consumer = %self.config.consumer_id,
                            "Claimed abandoned messages"
                        );
                    }
                }
            }
        }

        Ok(claimed)
    }

    /// Get stream information for monitoring.
    pub async fn get_stream_info(&self) -> Result<StreamInfo, StreamError> {
        let mut conn = (*self.redis).clone();

        let result: Result<redis::streams::StreamInfoStreamReply, _> = redis::cmd("XINFO")
            .arg("STREAM")
            .arg(&self.config.stream_name)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(info) => Ok(StreamInfo {
                stream_name: self.config.stream_name.clone(),
                length: info.length,
                first_entry_id: Some(info.first_entry.id.clone()),
                last_entry_id: Some(info.last_entry.id.clone()),
                groups: info.groups,
            }),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("no such key") || err_str.contains("ERR") {
                    // Stream doesn't exist yet
                    Ok(StreamInfo {
                        stream_name: self.config.stream_name.clone(),
                        length: 0,
                        first_entry_id: None,
                        last_entry_id: None,
                        groups: 0,
                    })
                } else {
                    Err(StreamError::Redis(e))
                }
            }
        }
    }
}

/// Stream information for monitoring.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Stream name.
    pub stream_name: String,
    /// Number of entries in the stream.
    pub length: usize,
    /// ID of the first entry.
    pub first_entry_id: Option<String>,
    /// ID of the last entry.
    pub last_entry_id: Option<String>,
    /// Number of consumer groups.
    pub groups: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_info() {
        let info = StreamInfo {
            stream_name: "test:stream".to_string(),
            length: 100,
            first_entry_id: Some("1-0".to_string()),
            last_entry_id: Some("100-0".to_string()),
            groups: 1,
        };

        assert_eq!(info.stream_name, "test:stream");
        assert_eq!(info.length, 100);
    }
}
