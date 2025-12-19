//! Redis stream producer for queueing jobs.
//!
//! This module provides the `StreamProducer` struct for adding
//! jobs to Redis streams.

use crate::error::StreamError;
use crate::registry::StreamDef;
use redis::aio::ConnectionManager;
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

/// Redis stream producer for queueing jobs.
///
/// This is used by APIs or services to add jobs to a stream
/// for background processing by workers.
///
/// # Example
///
/// ```rust,ignore
/// use stream_worker::StreamProducer;
///
/// let producer = StreamProducer::new(redis, "email:jobs");
///
/// // Queue a job
/// let job = EmailJob { to: "user@example.com", subject: "Hello" };
/// let message_id = producer.send(&job).await?;
/// ```
pub struct StreamProducer {
    redis: Arc<ConnectionManager>,
    stream_name: String,
    max_length: Option<i64>,
}

impl StreamProducer {
    /// Create a new producer for the given stream.
    pub fn new(redis: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            redis: Arc::new(redis),
            stream_name: stream_name.into(),
            max_length: None,
        }
    }

    /// Create a producer from a `StreamDef` implementation.
    pub fn from_stream_def<S: StreamDef>(redis: ConnectionManager) -> Self {
        Self {
            redis: Arc::new(redis),
            stream_name: S::STREAM_NAME.to_string(),
            max_length: Some(S::MAX_LENGTH),
        }
    }

    /// Set the maximum stream length (MAXLEN ~).
    pub fn with_max_length(mut self, max_length: i64) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Get the stream name.
    pub fn stream_name(&self) -> &str {
        &self.stream_name
    }

    /// Send a job to the stream.
    ///
    /// Returns the message ID assigned by Redis.
    pub async fn send<J: Serialize>(&self, job: &J) -> Result<String, StreamError> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        let message_id: String = if let Some(max_len) = self.max_length {
            // Use MAXLEN ~ for approximate trimming (more efficient)
            redis::cmd("XADD")
                .arg(&self.stream_name)
                .arg("MAXLEN")
                .arg("~")
                .arg(max_len)
                .arg("*")
                .arg("job")
                .arg(&job_json)
                .query_async(&mut conn)
                .await?
        } else {
            redis::cmd("XADD")
                .arg(&self.stream_name)
                .arg("*")
                .arg("job")
                .arg(&job_json)
                .query_async(&mut conn)
                .await?
        };

        debug!(
            stream = %self.stream_name,
            message_id = %message_id,
            "Added job to stream"
        );

        Ok(message_id)
    }

    /// Send a job with additional fields.
    ///
    /// This allows adding extra metadata alongside the job.
    pub async fn send_with_fields<J: Serialize>(
        &self,
        job: &J,
        fields: &[(&str, &str)],
    ) -> Result<String, StreamError> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        let mut cmd = redis::cmd("XADD");
        cmd.arg(&self.stream_name);

        if let Some(max_len) = self.max_length {
            cmd.arg("MAXLEN").arg("~").arg(max_len);
        }

        cmd.arg("*").arg("job").arg(&job_json);

        // Add extra fields
        for (key, value) in fields {
            cmd.arg(*key).arg(*value);
        }

        let message_id: String = cmd.query_async(&mut conn).await?;

        debug!(
            stream = %self.stream_name,
            message_id = %message_id,
            extra_fields = fields.len(),
            "Added job to stream with fields"
        );

        Ok(message_id)
    }

    /// Send multiple jobs in a pipeline for efficiency.
    pub async fn send_batch<J: Serialize>(&self, jobs: &[J]) -> Result<Vec<String>, StreamError> {
        if jobs.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = (*self.redis).clone();
        let mut pipe = redis::pipe();

        for job in jobs {
            let job_json = serde_json::to_string(job)?;

            if let Some(max_len) = self.max_length {
                pipe.cmd("XADD")
                    .arg(&self.stream_name)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg(max_len)
                    .arg("*")
                    .arg("job")
                    .arg(&job_json);
            } else {
                pipe.cmd("XADD")
                    .arg(&self.stream_name)
                    .arg("*")
                    .arg("job")
                    .arg(&job_json);
            }
        }

        let message_ids: Vec<String> = pipe.query_async(&mut conn).await?;

        debug!(
            stream = %self.stream_name,
            count = message_ids.len(),
            "Added batch of jobs to stream"
        );

        Ok(message_ids)
    }

    /// Get the current length of the stream.
    pub async fn stream_length(&self) -> Result<usize, StreamError> {
        let mut conn = (*self.redis).clone();

        let length: usize = redis::cmd("XLEN")
            .arg(&self.stream_name)
            .query_async(&mut conn)
            .await?;

        Ok(length)
    }
}

impl Clone for StreamProducer {
    fn clone(&self) -> Self {
        Self {
            redis: Arc::clone(&self.redis),
            stream_name: self.stream_name.clone(),
            max_length: self.max_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize)]
    struct TestJob {
        id: String,
        data: String,
    }

    struct TestStream;
    impl StreamDef for TestStream {
        const STREAM_NAME: &'static str = "test:jobs";
        const CONSUMER_GROUP: &'static str = "test_workers";
        const DLQ_STREAM: &'static str = "test:dlq";
        const MAX_LENGTH: i64 = 1000;
    }

    #[test]
    fn test_producer_stream_name() {
        // Note: We can't fully test without a Redis connection,
        // but we can test the configuration
        assert_eq!(TestStream::STREAM_NAME, "test:jobs");
        assert_eq!(TestStream::MAX_LENGTH, 1000);
    }
}
