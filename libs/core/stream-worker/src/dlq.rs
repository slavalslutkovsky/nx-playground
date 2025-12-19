//! Dead Letter Queue (DLQ) management module.
//!
//! This module provides functionality for managing and reprocessing
//! messages that have failed processing and been moved to the DLQ.
//!
//! ## Features
//!
//! - View DLQ statistics (message count, oldest message)
//! - List messages in the DLQ with pagination
//! - Reprocess individual or batches of messages
//! - Archive (delete) messages that should not be retried
//!
//! ## Usage
//!
//! ```rust,ignore
//! use stream_worker::dlq::DlqManager;
//!
//! let manager = DlqManager::new(redis, "email:jobs", "email:dlq");
//!
//! // Get stats
//! let stats = manager.stats().await?;
//!
//! // List messages
//! let messages = manager.list_messages(10, 0).await?;
//!
//! // Reprocess a batch
//! let count = manager.reprocess_batch(10).await?;
//! ```

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::error::StreamError;

/// DLQ statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqStats {
    /// Name of the DLQ stream.
    pub stream: String,
    /// Name of the source stream.
    pub source_stream: String,
    /// Total number of messages in the DLQ.
    pub message_count: usize,
    /// ID of the oldest message (if any).
    pub oldest_message_id: Option<String>,
    /// ID of the newest message (if any).
    pub newest_message_id: Option<String>,
}

/// A message in the DLQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMessage {
    /// Redis stream message ID.
    pub id: String,
    /// Original job payload (JSON string).
    pub job: String,
    /// Error message that caused the DLQ move.
    pub error: Option<String>,
    /// Timestamp when moved to DLQ (from message ID).
    pub timestamp_ms: Option<u64>,
}

/// Result of a reprocess operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReprocessResult {
    /// Number of messages successfully requeued.
    pub reprocessed: usize,
    /// Number of messages that failed to reprocess.
    pub failed: usize,
    /// IDs of messages that were reprocessed.
    pub message_ids: Vec<String>,
}

/// Manager for DLQ operations.
#[derive(Clone)]
pub struct DlqManager {
    redis: ConnectionManager,
    source_stream: String,
    dlq_stream: String,
    max_stream_length: i64,
}

impl DlqManager {
    /// Create a new DLQ manager.
    ///
    /// # Arguments
    ///
    /// * `redis` - Redis connection manager
    /// * `source_stream` - The original stream name (e.g., "email:jobs")
    /// * `dlq_stream` - The DLQ stream name (e.g., "email:dlq")
    pub fn new(
        redis: ConnectionManager,
        source_stream: impl Into<String>,
        dlq_stream: impl Into<String>,
    ) -> Self {
        Self {
            redis,
            source_stream: source_stream.into(),
            dlq_stream: dlq_stream.into(),
            max_stream_length: 100_000,
        }
    }

    /// Create a DLQ manager with custom max stream length.
    pub fn with_max_length(mut self, max_length: i64) -> Self {
        self.max_stream_length = max_length;
        self
    }

    /// Get DLQ statistics.
    pub async fn stats(&self) -> Result<DlqStats, StreamError> {
        let mut conn = self.redis.clone();

        // Get stream length
        let len: i64 = conn.xlen(&self.dlq_stream).await.unwrap_or(0);

        // Get first and last message IDs
        let (oldest, newest) = if len > 0 {
            // Get oldest (first) message
            let oldest_result: Value = redis::cmd("XRANGE")
                .arg(&self.dlq_stream)
                .arg("-")
                .arg("+")
                .arg("COUNT")
                .arg(1)
                .query_async(&mut conn)
                .await
                .unwrap_or(Value::Nil);

            // Get newest (last) message
            let newest_result: Value = redis::cmd("XREVRANGE")
                .arg(&self.dlq_stream)
                .arg("+")
                .arg("-")
                .arg("COUNT")
                .arg(1)
                .query_async(&mut conn)
                .await
                .unwrap_or(Value::Nil);

            (
                Self::extract_first_message_id(&oldest_result),
                Self::extract_first_message_id(&newest_result),
            )
        } else {
            (None, None)
        };

        Ok(DlqStats {
            stream: self.dlq_stream.clone(),
            source_stream: self.source_stream.clone(),
            message_count: len as usize,
            oldest_message_id: oldest,
            newest_message_id: newest,
        })
    }

    /// Extract the first message ID from an XRANGE/XREVRANGE response.
    fn extract_first_message_id(value: &Value) -> Option<String> {
        if let Value::Array(messages) = value {
            if let Some(Value::Array(msg)) = messages.first() {
                if let Some(Value::BulkString(id_bytes)) = msg.first() {
                    return String::from_utf8(id_bytes.clone()).ok();
                }
            }
        }
        None
    }

    /// List messages in the DLQ with pagination.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of messages to return
    /// * `offset` - Number of messages to skip (for pagination)
    pub async fn list_messages(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DlqMessage>, StreamError> {
        let mut conn = self.redis.clone();

        // We need to fetch offset + limit messages, then skip the first offset
        let fetch_count = offset + limit;

        let result: Value = redis::cmd("XRANGE")
            .arg(&self.dlq_stream)
            .arg("-")
            .arg("+")
            .arg("COUNT")
            .arg(fetch_count)
            .query_async(&mut conn)
            .await
            .map_err(StreamError::Redis)?;

        let messages = Self::parse_xrange_messages(&result);
        let result: Vec<DlqMessage> = messages
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(result)
    }

    /// Parse messages from an XRANGE response into DlqMessage structs.
    fn parse_xrange_messages(value: &Value) -> Vec<DlqMessage> {
        let mut result = Vec::new();

        if let Value::Array(messages) = value {
            for msg in messages {
                if let Value::Array(msg_parts) = msg {
                    if msg_parts.len() >= 2 {
                        // First element is the message ID
                        let id = match &msg_parts[0] {
                            Value::BulkString(bytes) => {
                                String::from_utf8(bytes.clone()).unwrap_or_default()
                            }
                            _ => continue,
                        };

                        // Second element is the fields array
                        let fields = Self::parse_fields(&msg_parts[1]);

                        let job = fields.get("job").cloned().unwrap_or_default();
                        let error = fields.get("error").cloned();

                        // Parse timestamp from message ID (format: timestamp-sequence)
                        let timestamp_ms = id.split('-').next().and_then(|ts| ts.parse::<u64>().ok());

                        result.push(DlqMessage {
                            id,
                            job,
                            error,
                            timestamp_ms,
                        });
                    }
                }
            }
        }

        result
    }

    /// Parse fields from a Redis stream message into a HashMap.
    fn parse_fields(value: &Value) -> HashMap<String, String> {
        let mut fields = HashMap::new();

        if let Value::Array(field_arr) = value {
            let mut iter = field_arr.iter();
            while let (Some(key), Some(val)) = (iter.next(), iter.next()) {
                let key_str = match key {
                    Value::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
                    _ => None,
                };
                let val_str = match val {
                    Value::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
                    Value::SimpleString(s) => Some(s.clone()),
                    _ => None,
                };

                if let (Some(k), Some(v)) = (key_str, val_str) {
                    fields.insert(k, v);
                }
            }
        }

        fields
    }

    /// Reprocess a single message by ID.
    ///
    /// Moves the message from DLQ back to the source stream.
    pub async fn reprocess_message(&self, message_id: &str) -> Result<bool, StreamError> {
        let mut conn = self.redis.clone();

        // Read the specific message
        let result: Value = redis::cmd("XRANGE")
            .arg(&self.dlq_stream)
            .arg(message_id)
            .arg(message_id)
            .query_async(&mut conn)
            .await
            .map_err(StreamError::Redis)?;

        let messages = Self::parse_xrange_messages(&result);
        let Some(msg) = messages.into_iter().next() else {
            warn!(message_id = %message_id, "Message not found in DLQ");
            return Ok(false);
        };

        // Extract job data
        let job_data = if msg.job.is_empty() {
            return Err(StreamError::Processing(
                "No job data in DLQ message".to_string(),
            ));
        } else {
            msg.job
        };

        // Add back to source stream
        let _: String = redis::cmd("XADD")
            .arg(&self.source_stream)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.max_stream_length)
            .arg("*")
            .arg("job")
            .arg(&job_data)
            .query_async(&mut conn)
            .await
            .map_err(StreamError::Redis)?;

        // Delete from DLQ
        let deleted: i64 = conn
            .xdel(&self.dlq_stream, &[message_id])
            .await
            .map_err(StreamError::Redis)?;

        if deleted > 0 {
            info!(
                message_id = %message_id,
                source_stream = %self.source_stream,
                "Reprocessed message from DLQ"
            );
        }

        Ok(deleted > 0)
    }

    /// Reprocess a batch of messages from the DLQ.
    ///
    /// Takes the oldest messages first.
    ///
    /// # Arguments
    ///
    /// * `count` - Maximum number of messages to reprocess
    pub async fn reprocess_batch(&self, count: usize) -> Result<ReprocessResult, StreamError> {
        let messages = self.list_messages(count, 0).await?;

        let mut reprocessed = 0;
        let mut failed = 0;
        let mut message_ids = Vec::new();

        for msg in messages {
            match self.reprocess_message(&msg.id).await {
                Ok(true) => {
                    reprocessed += 1;
                    message_ids.push(msg.id);
                }
                Ok(false) => {
                    failed += 1;
                }
                Err(e) => {
                    warn!(message_id = %msg.id, error = %e, "Failed to reprocess message");
                    failed += 1;
                }
            }
        }

        info!(
            reprocessed = reprocessed,
            failed = failed,
            dlq_stream = %self.dlq_stream,
            "Batch reprocess completed"
        );

        Ok(ReprocessResult {
            reprocessed,
            failed,
            message_ids,
        })
    }

    /// Archive (delete) a message from the DLQ.
    ///
    /// Use this when a message should not be retried.
    pub async fn archive_message(&self, message_id: &str) -> Result<bool, StreamError> {
        let mut conn = self.redis.clone();

        let deleted: i64 = conn
            .xdel(&self.dlq_stream, &[message_id])
            .await
            .map_err(|e| StreamError::Redis(e))?;

        if deleted > 0 {
            info!(
                message_id = %message_id,
                dlq_stream = %self.dlq_stream,
                "Archived message from DLQ"
            );
        }

        Ok(deleted > 0)
    }

    /// Archive all messages in the DLQ.
    ///
    /// WARNING: This permanently deletes all DLQ messages.
    pub async fn archive_all(&self) -> Result<usize, StreamError> {
        let mut conn = self.redis.clone();

        // Get current length before deletion
        let len: i64 = conn.xlen(&self.dlq_stream).await.unwrap_or(0);

        if len > 0 {
            // Delete the stream entirely
            let _: () = redis::cmd("DEL")
                .arg(&self.dlq_stream)
                .query_async(&mut conn)
                .await
                .map_err(|e| StreamError::Redis(e))?;

            info!(
                count = len,
                dlq_stream = %self.dlq_stream,
                "Archived all messages from DLQ"
            );
        }

        Ok(len as usize)
    }

    /// Get the source stream name.
    pub fn source_stream(&self) -> &str {
        &self.source_stream
    }

    /// Get the DLQ stream name.
    pub fn dlq_stream(&self) -> &str {
        &self.dlq_stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlq_stats_serialization() {
        let stats = DlqStats {
            stream: "test:dlq".to_string(),
            source_stream: "test:jobs".to_string(),
            message_count: 5,
            oldest_message_id: Some("1234-0".to_string()),
            newest_message_id: Some("5678-0".to_string()),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"message_count\":5"));
        assert!(json.contains("\"stream\":\"test:dlq\""));
    }

    #[test]
    fn test_dlq_message_serialization() {
        let msg = DlqMessage {
            id: "1234-0".to_string(),
            job: r#"{"id":"test"}"#.to_string(),
            error: Some("Test error".to_string()),
            timestamp_ms: Some(1234567890),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"id\":\"1234-0\""));
        assert!(json.contains("\"error\":\"Test error\""));
    }

    #[test]
    fn test_reprocess_result_serialization() {
        let result = ReprocessResult {
            reprocessed: 3,
            failed: 1,
            message_ids: vec!["1-0".to_string(), "2-0".to_string(), "3-0".to_string()],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"reprocessed\":3"));
        assert!(json.contains("\"failed\":1"));
    }
}
