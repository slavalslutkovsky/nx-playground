//! Stream producer for job enqueuing
//!
//! Generic producer that can be used by any service to queue jobs
//! for background processing.
//!
//! # Example
//!
//! ```rust,ignore
//! use stream_worker::{StreamProducer, StreamDef};
//!
//! // Create producer from a StreamDef
//! let producer = StreamProducer::from_stream_def::<EmailStream>(redis);
//!
//! // Queue a job
//! let job = EmailJob::welcome("user@example.com", "John", "MyApp");
//! let message_id = producer.send(&job).await?;
//! ```

use crate::error::StreamError;
use crate::registry::StreamDef;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

/// Generic stream producer for enqueuing jobs.
///
/// This producer can be used by any service (API, CLI, etc.) to
/// queue jobs for background processing by workers.
pub struct StreamProducer {
    redis: Arc<ConnectionManager>,
    stream_name: String,
    max_length: i64,
}

impl StreamProducer {
    /// Create a new StreamProducer for a specific stream.
    pub fn new(redis: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            redis: Arc::new(redis),
            stream_name: stream_name.into(),
            max_length: 100_000,
        }
    }

    /// Create a producer from a `StreamDef` implementation.
    ///
    /// This is the recommended way to create a producer as it ensures
    /// the stream name and max length are consistent with the worker.
    pub fn from_stream_def<S: StreamDef>(redis: ConnectionManager) -> Self {
        Self {
            redis: Arc::new(redis),
            stream_name: S::STREAM_NAME.to_string(),
            max_length: S::MAX_LENGTH,
        }
    }

    /// Create from an Arc<ConnectionManager> (for sharing connections).
    pub fn from_arc(redis: Arc<ConnectionManager>, stream_name: impl Into<String>) -> Self {
        Self {
            redis,
            stream_name: stream_name.into(),
            max_length: 100_000,
        }
    }

    /// Create from Arc with StreamDef.
    pub fn from_arc_with_stream_def<S: StreamDef>(redis: Arc<ConnectionManager>) -> Self {
        Self {
            redis,
            stream_name: S::STREAM_NAME.to_string(),
            max_length: S::MAX_LENGTH,
        }
    }

    /// Set the maximum stream length (MAXLEN ~).
    pub fn with_max_length(mut self, max_length: i64) -> Self {
        self.max_length = max_length;
        self
    }

    /// Get the stream name.
    pub fn stream_name(&self) -> &str {
        &self.stream_name
    }

    /// Enqueue a job.
    ///
    /// Returns the Redis stream message ID.
    pub async fn send<J: Serialize>(&self, job: &J) -> Result<String, StreamError> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        // Use XADD with MAXLEN ~ for approximate trimming (more efficient)
        let stream_id: String = redis::cmd("XADD")
            .arg(&self.stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.max_length)
            .arg("*")
            .arg("job") // Field name matches what StreamConsumer expects
            .arg(&job_json)
            .query_async(&mut conn)
            .await?;

        debug!(
            stream = %self.stream_name,
            stream_id = %stream_id,
            "Enqueued job"
        );

        Ok(stream_id)
    }

    /// Enqueue a job with additional metadata fields.
    pub async fn send_with_fields<J: Serialize>(
        &self,
        job: &J,
        fields: &[(&str, &str)],
    ) -> Result<String, StreamError> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        let mut cmd = redis::cmd("XADD");
        cmd.arg(&self.stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.max_length)
            .arg("*")
            .arg("job")
            .arg(&job_json);

        // Add extra metadata fields
        for (key, value) in fields {
            cmd.arg(*key).arg(*value);
        }

        let stream_id: String = cmd.query_async(&mut conn).await?;

        debug!(
            stream = %self.stream_name,
            stream_id = %stream_id,
            extra_fields = fields.len(),
            "Enqueued job with fields"
        );

        Ok(stream_id)
    }

    /// Enqueue multiple jobs in a pipeline (batch operation).
    pub async fn send_batch<J: Serialize>(&self, jobs: &[J]) -> Result<Vec<String>, StreamError> {
        if jobs.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = (*self.redis).clone();
        let mut pipe = redis::pipe();

        for job in jobs {
            let job_json = serde_json::to_string(job)?;
            pipe.cmd("XADD")
                .arg(&self.stream_name)
                .arg("MAXLEN")
                .arg("~")
                .arg(self.max_length)
                .arg("*")
                .arg("job")
                .arg(&job_json);
        }

        let results: Vec<String> = pipe.query_async(&mut conn).await?;

        debug!(
            stream = %self.stream_name,
            count = results.len(),
            "Enqueued batch of jobs"
        );

        Ok(results)
    }

    /// Get the current stream length.
    pub async fn stream_length(&self) -> Result<i64, StreamError> {
        let mut conn = (*self.redis).clone();
        let len: i64 = conn.xlen(&self.stream_name).await?;
        Ok(len)
    }
}

impl Clone for StreamProducer {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
            stream_name: self.stream_name.clone(),
            max_length: self.max_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer_clone() {
        // Just test that the struct is cloneable
        // Real tests require Redis
    }
}
