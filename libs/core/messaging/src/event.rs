//! Job event wrapper with metadata.

use crate::job::Job;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A job event with associated metadata.
///
/// This wraps a job with additional information like the message ID,
/// delivery count, and timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "J: for<'a> Deserialize<'a>"))]
pub struct JobEvent<J: Job> {
    /// The underlying job
    pub job: J,

    /// Backend-specific message ID (e.g., NATS sequence)
    pub message_id: String,

    /// Number of times this message has been delivered
    pub delivery_count: u32,

    /// When the job was first created
    pub created_at: DateTime<Utc>,

    /// When this delivery attempt started
    pub delivered_at: DateTime<Utc>,
}

impl<J: Job> JobEvent<J> {
    /// Create a new job event.
    pub fn new(job: J, message_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            job,
            message_id: message_id.into(),
            delivery_count: 1,
            created_at: now,
            delivered_at: now,
        }
    }

    /// Create a job event with delivery count (for redeliveries).
    pub fn with_delivery_count(job: J, message_id: impl Into<String>, delivery_count: u32) -> Self {
        Self {
            job,
            message_id: message_id.into(),
            delivery_count,
            created_at: Utc::now(),
            delivered_at: Utc::now(),
        }
    }

    /// Get the job ID.
    pub fn job_id(&self) -> String {
        self.job.job_id()
    }

    /// Get the retry count from the job.
    pub fn retry_count(&self) -> u32 {
        self.job.retry_count()
    }

    /// Check if this is a redelivery.
    pub fn is_redelivery(&self) -> bool {
        self.delivery_count > 1
    }

    /// Check if the job can be retried.
    pub fn can_retry(&self) -> bool {
        self.job.can_retry()
    }

    /// Get the job reference.
    pub fn job(&self) -> &J {
        &self.job
    }

    /// Take ownership of the job.
    pub fn into_job(self) -> J {
        self.job
    }
}

/// Result of processing a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessResult {
    /// Job processed successfully
    Success {
        /// Processing duration in milliseconds
        duration_ms: u64,
    },

    /// Job failed and should be retried
    Retry {
        /// Error message
        error: String,
        /// Delay before retry in milliseconds
        delay_ms: u64,
    },

    /// Job failed permanently and should go to DLQ
    DeadLetter {
        /// Error message
        error: String,
    },

    /// Job skipped (e.g., duplicate, expired)
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

impl ProcessResult {
    /// Create a success result.
    pub fn success(duration_ms: u64) -> Self {
        Self::Success { duration_ms }
    }

    /// Create a retry result.
    pub fn retry(error: impl Into<String>, delay_ms: u64) -> Self {
        Self::Retry {
            error: error.into(),
            delay_ms,
        }
    }

    /// Create a dead letter result.
    pub fn dead_letter(error: impl Into<String>) -> Self {
        Self::DeadLetter {
            error: error.into(),
        }
    }

    /// Create a skipped result.
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self::Skipped {
            reason: reason.into(),
        }
    }

    /// Check if this is a success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if this should be retried.
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Retry { .. })
    }

    /// Check if this should go to DLQ.
    pub fn is_dead_letter(&self) -> bool {
        matches!(self, Self::DeadLetter { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    struct TestJob {
        id: String,
        retry_count: u32,
    }

    impl Job for TestJob {
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
    fn test_job_event() {
        let job = TestJob {
            id: "job-1".to_string(),
            retry_count: 0,
        };

        let event = JobEvent::new(job, "msg-123");

        assert_eq!(event.job_id(), "job-1");
        assert_eq!(event.message_id, "msg-123");
        assert_eq!(event.delivery_count, 1);
        assert!(!event.is_redelivery());
    }

    #[test]
    fn test_job_event_redelivery() {
        let job = TestJob {
            id: "job-2".to_string(),
            retry_count: 1,
        };

        let event = JobEvent::with_delivery_count(job, "msg-456", 3);

        assert!(event.is_redelivery());
        assert_eq!(event.delivery_count, 3);
        assert_eq!(event.retry_count(), 1);
    }

    #[test]
    fn test_process_result() {
        let success = ProcessResult::success(150);
        assert!(success.is_success());

        let retry = ProcessResult::retry("timeout", 1000);
        assert!(retry.should_retry());

        let dead = ProcessResult::dead_letter("invalid email");
        assert!(dead.is_dead_letter());
    }

    #[test]
    fn test_process_result_serialization() {
        let success = ProcessResult::success(150);
        let json = serde_json::to_string(&success).unwrap();
        assert!(json.contains("\"type\":\"success\""));
    }
}
