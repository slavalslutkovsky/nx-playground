//! Stream event wrapper
//!
//! Wraps a job with its stream metadata (ID, timestamp, etc.)

use crate::registry::StreamJob;
use chrono::{DateTime, Utc};

/// A stream event containing job data and metadata
#[derive(Debug, Clone)]
pub struct StreamEvent<J: StreamJob> {
    /// Redis stream entry ID (e.g., "1234567890123-0")
    pub stream_id: String,

    /// The job payload
    pub job: J,

    /// When the event was created (parsed from stream ID)
    pub timestamp: DateTime<Utc>,

    /// Number of times this message has been delivered
    pub delivery_count: u32,
}

impl<J: StreamJob> StreamEvent<J> {
    /// Create a new StreamEvent
    pub fn new(stream_id: String, job: J) -> Self {
        let timestamp = Self::parse_timestamp(&stream_id);
        Self {
            stream_id,
            job,
            timestamp,
            delivery_count: 1,
        }
    }

    /// Create a StreamEvent with delivery count
    pub fn with_delivery_count(stream_id: String, job: J, delivery_count: u32) -> Self {
        let timestamp = Self::parse_timestamp(&stream_id);
        Self {
            stream_id,
            job,
            timestamp,
            delivery_count,
        }
    }

    /// Parse timestamp from Redis stream ID
    ///
    /// Stream IDs are in format "timestamp_ms-sequence"
    fn parse_timestamp(stream_id: &str) -> DateTime<Utc> {
        stream_id
            .split('-')
            .next()
            .and_then(|ts| ts.parse::<i64>().ok())
            .and_then(DateTime::from_timestamp_millis)
            .unwrap_or_else(Utc::now)
    }

    /// Get the job ID
    pub fn job_id(&self) -> String {
        self.job.job_id()
    }

    /// Get the job's retry count
    pub fn retry_count(&self) -> u32 {
        self.job.retry_count()
    }

    /// Check if this is a redelivery
    pub fn is_redelivery(&self) -> bool {
        self.delivery_count > 1
    }

    /// Get how long ago the event was created
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.timestamp
    }

    /// Get age in milliseconds
    pub fn age_ms(&self) -> i64 {
        self.age().num_milliseconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize, Debug)]
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

    #[test]
    fn test_parse_timestamp() {
        let job = TestJob {
            id: "test-1".to_string(),
            retry_count: 0,
        };

        // Use a recent timestamp
        let now_ms = Utc::now().timestamp_millis();
        let stream_id = format!("{}-0", now_ms);

        let event = StreamEvent::new(stream_id, job);

        // Timestamp should be close to now
        assert!(event.age_ms() < 1000);
        assert!(!event.is_redelivery());
    }

    #[test]
    fn test_redelivery() {
        let job = TestJob {
            id: "test-1".to_string(),
            retry_count: 1,
        };

        let event = StreamEvent::with_delivery_count("1234567890123-0".to_string(), job, 3);

        assert!(event.is_redelivery());
        assert_eq!(event.delivery_count, 3);
        assert_eq!(event.retry_count(), 1);
    }
}
