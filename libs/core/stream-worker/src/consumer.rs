//! Stream consumer for Redis operations
//!
//! Handles reading messages from Redis streams using consumer groups.

use crate::config::WorkerConfig;
use crate::error::StreamError;
use crate::event::StreamEvent;
use crate::registry::StreamJob;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Stream consumer for Redis operations
pub struct StreamConsumer {
    redis: Arc<ConnectionManager>,
    config: WorkerConfig,
}

impl StreamConsumer {
    /// Create a new StreamConsumer
    pub fn new(redis: Arc<ConnectionManager>, config: WorkerConfig) -> Self {
        Self { redis, config }
    }

    /// Get a reference to the Redis connection
    pub fn redis(&self) -> Arc<ConnectionManager> {
        self.redis.clone()
    }

    /// Get the stream name
    pub fn stream_name(&self) -> &str {
        &self.config.stream_name
    }

    /// Get the consumer group
    pub fn consumer_group(&self) -> &str {
        &self.config.consumer_group
    }

    /// Get the consumer ID
    pub fn consumer_id(&self) -> &str {
        &self.config.consumer_id
    }

    /// Initialize the consumer group if it doesn't exist
    pub async fn init_consumer_group(&self) -> Result<(), StreamError> {
        let mut conn = (*self.redis).clone();

        // Try to create the group, ignore error if it already exists
        let result: RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("0") // Start from beginning
            .arg("MKSTREAM") // Create stream if it doesn't exist
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                info!(
                    stream = %self.config.stream_name,
                    group = %self.config.consumer_group,
                    "Created consumer group"
                );
            }
            Err(e) if e.to_string().contains("BUSYGROUP") => {
                debug!(
                    stream = %self.config.stream_name,
                    group = %self.config.consumer_group,
                    "Consumer group already exists"
                );
            }
            Err(e) => return Err(StreamError::Redis(e)),
        }

        Ok(())
    }

    /// Read pending messages (messages that were delivered but not acknowledged)
    pub async fn read_pending<J: StreamJob + DeserializeOwned>(
        &self,
        count: usize,
    ) -> Result<Vec<StreamEvent<J>>, StreamError> {
        let mut conn = (*self.redis).clone();

        // Read pending messages for this consumer
        let result: RedisResult<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> =
            redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(&self.config.consumer_group)
                .arg(&self.config.consumer_id)
                .arg("COUNT")
                .arg(count)
                .arg("STREAMS")
                .arg(&self.config.stream_name)
                .arg("0") // Read pending messages
                .query_async(&mut conn)
                .await;

        match result {
            Ok(streams) => self.parse_stream_response(streams),
            Err(e) if e.to_string().contains("NOGROUP") => {
                // Consumer group doesn't exist yet
                Ok(vec![])
            }
            Err(e) => Err(StreamError::Redis(e)),
        }
    }

    /// Read new messages from the stream
    pub async fn read_new<J: StreamJob + DeserializeOwned>(
        &self,
        count: usize,
    ) -> Result<Vec<StreamEvent<J>>, StreamError> {
        let mut conn = (*self.redis).clone();

        // Build the command with optional blocking
        let mut cmd = redis::cmd("XREADGROUP");
        cmd.arg("GROUP")
            .arg(&self.config.consumer_group)
            .arg(&self.config.consumer_id);

        if let Some(timeout) = self.config.blocking_timeout_ms {
            cmd.arg("BLOCK").arg(timeout);
        }

        cmd.arg("COUNT")
            .arg(count)
            .arg("STREAMS")
            .arg(&self.config.stream_name)
            .arg(">"); // Only new messages

        let result: RedisResult<Option<Vec<(String, Vec<(String, Vec<(String, String)>)>)>>> =
            cmd.query_async(&mut conn).await;

        match result {
            Ok(Some(streams)) => self.parse_stream_response(streams),
            Ok(None) => Ok(vec![]), // No messages (blocking timeout)
            Err(e) if e.to_string().contains("NOGROUP") => {
                // Consumer group doesn't exist yet
                Ok(vec![])
            }
            Err(e) => Err(StreamError::Redis(e)),
        }
    }

    /// Acknowledge a message
    pub async fn ack(&self, stream_id: &str) -> Result<(), StreamError> {
        let mut conn = (*self.redis).clone();

        let _: i64 = redis::cmd("XACK")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(stream_id)
            .query_async(&mut conn)
            .await?;

        debug!(stream_id = %stream_id, "Acknowledged message");
        Ok(())
    }

    /// Claim abandoned messages from other consumers
    pub async fn claim_abandoned<J: StreamJob + DeserializeOwned>(
        &self,
        count: usize,
    ) -> Result<Vec<StreamEvent<J>>, StreamError> {
        let mut conn = (*self.redis).clone();

        // First, get pending entries info
        let pending: RedisResult<Vec<(String, String, i64, i64)>> = redis::cmd("XPENDING")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("-")
            .arg("+")
            .arg(count)
            .query_async(&mut conn)
            .await;

        let pending = match pending {
            Ok(p) => p,
            Err(e) if e.to_string().contains("NOGROUP") => return Ok(vec![]),
            Err(e) => return Err(StreamError::Redis(e)),
        };

        if pending.is_empty() {
            return Ok(vec![]);
        }

        // Filter for messages that are old enough to claim
        let claim_ids: Vec<String> = pending
            .iter()
            .filter(|(_, _, idle_time, _)| *idle_time > self.config.claim_timeout_ms as i64)
            .map(|(id, _, _, _)| id.clone())
            .collect();

        if claim_ids.is_empty() {
            return Ok(vec![]);
        }

        // Claim the messages
        let mut cmd = redis::cmd("XCLAIM");
        cmd.arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg(&self.config.consumer_id)
            .arg(self.config.claim_timeout_ms);

        for id in &claim_ids {
            cmd.arg(id);
        }

        let result: RedisResult<Vec<(String, Vec<(String, String)>)>> =
            cmd.query_async(&mut conn).await;

        match result {
            Ok(entries) => {
                let events = self.parse_entries(entries)?;
                if !events.is_empty() {
                    warn!(count = events.len(), "Claimed abandoned messages");
                }
                Ok(events)
            }
            Err(e) => Err(StreamError::Redis(e)),
        }
    }

    /// Get stream info (length, groups, etc.)
    pub async fn stream_info(&self) -> Result<StreamInfo, StreamError> {
        let mut conn = (*self.redis).clone();

        let len: i64 = conn.xlen(&self.config.stream_name).await?;

        // Get pending count for this consumer group
        let pending: RedisResult<(i64, Option<String>, Option<String>, Option<Vec<(String, i64)>>)> =
            redis::cmd("XPENDING")
                .arg(&self.config.stream_name)
                .arg(&self.config.consumer_group)
                .query_async(&mut conn)
                .await;

        let pending_count = pending.map(|(count, _, _, _)| count).unwrap_or(0);

        Ok(StreamInfo {
            stream_name: self.config.stream_name.clone(),
            length: len,
            pending_count,
            consumer_group: self.config.consumer_group.clone(),
        })
    }

    /// Parse stream response from XREADGROUP
    fn parse_stream_response<J: StreamJob + DeserializeOwned>(
        &self,
        streams: Vec<(String, Vec<(String, Vec<(String, String)>)>)>,
    ) -> Result<Vec<StreamEvent<J>>, StreamError> {
        let mut events = Vec::new();

        for (_stream_name, entries) in streams {
            let parsed = self.parse_entries(entries)?;
            events.extend(parsed);
        }

        Ok(events)
    }

    /// Parse entries from Redis response
    fn parse_entries<J: StreamJob + DeserializeOwned>(
        &self,
        entries: Vec<(String, Vec<(String, String)>)>,
    ) -> Result<Vec<StreamEvent<J>>, StreamError> {
        let mut events = Vec::new();

        for (stream_id, fields) in entries {
            // Find the "job" field (main stream format)
            let job_data = fields
                .iter()
                .find(|(k, _)| k == "job")
                .map(|(_, v)| v.as_str());

            if let Some(json) = job_data {
                match serde_json::from_str::<J>(json) {
                    Ok(job) => {
                        events.push(StreamEvent::new(stream_id, job));
                    }
                    Err(e) => {
                        warn!(
                            stream_id = %stream_id,
                            error = %e,
                            "Failed to parse job, skipping"
                        );
                    }
                }
            } else {
                warn!(
                    stream_id = %stream_id,
                    fields = ?fields.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
                    "Missing 'job' field in message"
                );
            }
        }

        Ok(events)
    }
}

/// Stream information
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_name: String,
    pub length: i64,
    pub pending_count: i64,
    pub consumer_group: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_info() {
        let info = StreamInfo {
            stream_name: "test:stream".to_string(),
            length: 100,
            pending_count: 5,
            consumer_group: "test:group".to_string(),
        };

        assert_eq!(info.length, 100);
        assert_eq!(info.pending_count, 5);
    }
}
