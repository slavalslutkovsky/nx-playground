//! Dead Letter Queue (DLQ) management
//!
//! Handles failed jobs that have exceeded their retry limits.

use crate::error::StreamError;
use crate::registry::StreamJob;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

// Type alias for Redis stream entries
type StreamEntries = Vec<(String, Vec<(String, String)>)>;

/// Dead Letter Queue manager
pub struct DlqManager {
    redis: Arc<ConnectionManager>,
    dlq_stream: String,
    max_length: i64,
}

impl DlqManager {
    /// Create a new DlqManager
    pub fn new(redis: Arc<ConnectionManager>, dlq_stream: impl Into<String>) -> Self {
        Self {
            redis,
            dlq_stream: dlq_stream.into(),
            max_length: 10_000,
        }
    }

    /// Set the maximum DLQ length
    pub fn with_max_length(mut self, max_length: i64) -> Self {
        self.max_length = max_length;
        self
    }

    /// Get the DLQ stream name
    pub fn dlq_stream(&self) -> &str {
        &self.dlq_stream
    }

    /// Move a job to the dead letter queue
    pub async fn move_to_dlq<J: StreamJob + Serialize>(
        &self,
        job: &J,
        error: &str,
        original_stream_id: &str,
    ) -> Result<String, StreamError> {
        let entry = DlqEntry {
            job_id: job.job_id(),
            job_data: serde_json::to_value(job)?,
            error: error.to_string(),
            original_stream_id: original_stream_id.to_string(),
            retry_count: job.retry_count(),
            failed_at: Utc::now(),
        };

        let data = serde_json::to_string(&entry)?;
        let mut conn = (*self.redis).clone();

        let dlq_id: String = redis::cmd("XADD")
            .arg(&self.dlq_stream)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.max_length)
            .arg("*")
            .arg("data")
            .arg(&data)
            .query_async(&mut conn)
            .await?;

        info!(
            job_id = %job.job_id(),
            dlq_id = %dlq_id,
            error = %error,
            retry_count = job.retry_count(),
            "Moved job to DLQ"
        );

        Ok(dlq_id)
    }

    /// Get DLQ statistics
    pub async fn stats(&self) -> Result<DlqStats, StreamError> {
        let mut conn = (*self.redis).clone();

        let len: i64 = conn.xlen(&self.dlq_stream).await.unwrap_or(0);

        // Get oldest and newest entries
        let oldest: Option<StreamEntries> = redis::cmd("XRANGE")
            .arg(&self.dlq_stream)
            .arg("-")
            .arg("+")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut conn)
            .await
            .ok();

        let newest: Option<StreamEntries> = redis::cmd("XREVRANGE")
            .arg(&self.dlq_stream)
            .arg("+")
            .arg("-")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut conn)
            .await
            .ok();

        let oldest_id = oldest.and_then(|v| v.first().map(|(id, _)| id.clone()));
        let newest_id = newest.and_then(|v| v.first().map(|(id, _)| id.clone()));

        Ok(DlqStats {
            stream_name: self.dlq_stream.clone(),
            length: len,
            oldest_entry_id: oldest_id,
            newest_entry_id: newest_id,
        })
    }

    /// List DLQ entries
    pub async fn list(
        &self,
        count: usize,
        offset: Option<&str>,
    ) -> Result<Vec<DlqEntry>, StreamError> {
        let mut conn = (*self.redis).clone();

        let start = offset.unwrap_or("-");

        let entries: StreamEntries = redis::cmd("XRANGE")
            .arg(&self.dlq_stream)
            .arg(start)
            .arg("+")
            .arg("COUNT")
            .arg(count)
            .query_async(&mut conn)
            .await?;

        let mut results = Vec::new();

        for (_id, fields) in entries {
            if let Some(data) = fields.iter().find(|(k, _)| k == "data").map(|(_, v)| v)
                && let Ok(entry) = serde_json::from_str::<DlqEntry>(data)
            {
                results.push(entry);
            }
        }

        Ok(results)
    }

    /// Get a specific DLQ entry by ID
    pub async fn get(&self, dlq_id: &str) -> Result<Option<DlqEntry>, StreamError> {
        let mut conn = (*self.redis).clone();

        let entries: StreamEntries = redis::cmd("XRANGE")
            .arg(&self.dlq_stream)
            .arg(dlq_id)
            .arg(dlq_id)
            .query_async(&mut conn)
            .await?;

        if let Some((_id, fields)) = entries.first()
            && let Some(data) = fields.iter().find(|(k, _)| k == "data").map(|(_, v)| v)
        {
            return Ok(serde_json::from_str(data).ok());
        }

        Ok(None)
    }

    /// Delete an entry from the DLQ
    pub async fn delete(&self, dlq_id: &str) -> Result<bool, StreamError> {
        let mut conn = (*self.redis).clone();

        let deleted: i64 = conn.xdel(&self.dlq_stream, &[dlq_id]).await?;

        debug!(dlq_id = %dlq_id, "Deleted DLQ entry");

        Ok(deleted > 0)
    }

    /// Purge all entries from the DLQ
    pub async fn purge(&self) -> Result<i64, StreamError> {
        let mut conn = (*self.redis).clone();

        // Get current length
        let len: i64 = conn.xlen(&self.dlq_stream).await?;

        if len > 0 {
            // Trim to 0
            let _: () = redis::cmd("XTRIM")
                .arg(&self.dlq_stream)
                .arg("MAXLEN")
                .arg(0)
                .query_async(&mut conn)
                .await?;

            info!(count = len, "Purged DLQ");
        }

        Ok(len)
    }
}

impl Clone for DlqManager {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
            dlq_stream: self.dlq_stream.clone(),
            max_length: self.max_length,
        }
    }
}

/// DLQ entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqEntry {
    /// Original job ID
    pub job_id: String,

    /// Serialized job data
    pub job_data: serde_json::Value,

    /// Error message that caused the failure
    pub error: String,

    /// Original stream entry ID
    pub original_stream_id: String,

    /// Number of retry attempts
    pub retry_count: u32,

    /// When the job was moved to DLQ
    pub failed_at: DateTime<Utc>,
}

/// DLQ statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqStats {
    pub stream_name: String,
    pub length: i64,
    pub oldest_entry_id: Option<String>,
    pub newest_entry_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlq_entry_serialization() {
        let entry = DlqEntry {
            job_id: "job-1".to_string(),
            job_data: serde_json::json!({"test": "data"}),
            error: "Test error".to_string(),
            original_stream_id: "1234567890123-0".to_string(),
            retry_count: 3,
            failed_at: Utc::now(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DlqEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.job_id, "job-1");
        assert_eq!(deserialized.retry_count, 3);
    }
}
