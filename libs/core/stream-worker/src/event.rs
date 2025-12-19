//! Stream event wrapper for domain payloads.
//!
//! This module provides a generic event wrapper that adds metadata
//! (action, retry count, timestamps) around domain-specific payloads.

use crate::registry::Action;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic stream event wrapper.
///
/// This wraps a domain-specific payload with common metadata
/// for stream processing.
///
/// # Type Parameters
///
/// * `T` - The domain-specific payload type (must be Serialize + Deserialize)
///
/// # Example
///
/// ```rust,ignore
/// use stream_worker::{StreamEvent, Action};
///
/// #[derive(Clone, Serialize, Deserialize)]
/// struct TaskPayload {
///     task_id: Uuid,
///     title: String,
/// }
///
/// let event = StreamEvent::new(
///     Action::Create,
///     TaskPayload { task_id: Uuid::new_v4(), title: "My Task".to_string() },
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent<T> {
    /// Unique event ID.
    pub id: Uuid,
    /// The action being performed.
    pub action: Action,
    /// The domain-specific payload.
    pub payload: T,
    /// Optional request ID for correlation/tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Current retry count (0 for first attempt).
    pub retry_count: u32,
    /// Timestamp when the event was created.
    pub created_at: DateTime<Utc>,
}

impl<T> StreamEvent<T> {
    /// Create a new stream event with the given action and payload.
    pub fn new(action: Action, payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            payload,
            request_id: None,
            retry_count: 0,
            created_at: Utc::now(),
        }
    }

    /// Create a new stream event with a request ID for correlation.
    pub fn with_request_id(action: Action, payload: T, request_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            payload,
            request_id: Some(request_id.into()),
            retry_count: 0,
            created_at: Utc::now(),
        }
    }

    /// Set the request ID.
    pub fn set_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Check if max retries exceeded.
    pub fn exceeded_max_retries(&self, max_retries: u32) -> bool {
        self.retry_count >= max_retries
    }
}

impl<T: Clone> StreamEvent<T> {
    /// Create a new event with incremented retry count.
    pub fn with_retry(&self) -> Self {
        Self {
            id: self.id,
            action: self.action,
            payload: self.payload.clone(),
            request_id: self.request_id.clone(),
            retry_count: self.retry_count + 1,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestPayload {
        message: String,
    }

    #[test]
    fn test_stream_event_new() {
        let payload = TestPayload {
            message: "test".to_string(),
        };
        let event = StreamEvent::new(Action::Create, payload);

        assert_eq!(event.action, Action::Create);
        assert_eq!(event.payload.message, "test");
        assert_eq!(event.retry_count, 0);
        assert!(event.request_id.is_none());
    }

    #[test]
    fn test_stream_event_with_request_id() {
        let payload = TestPayload {
            message: "test".to_string(),
        };
        let event = StreamEvent::with_request_id(Action::Update, payload, "req-123");

        assert_eq!(event.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_stream_event_with_retry() {
        let payload = TestPayload {
            message: "test".to_string(),
        };
        let event = StreamEvent::new(Action::Create, payload);
        assert_eq!(event.retry_count, 0);

        let retry1 = event.with_retry();
        assert_eq!(retry1.retry_count, 1);
        assert_eq!(retry1.id, event.id); // Same ID

        let retry2 = retry1.with_retry();
        assert_eq!(retry2.retry_count, 2);
    }

    #[test]
    fn test_exceeded_max_retries() {
        let payload = TestPayload {
            message: "test".to_string(),
        };
        let mut event = StreamEvent::new(Action::Create, payload);

        assert!(!event.exceeded_max_retries(3));

        event.retry_count = 3;
        assert!(event.exceeded_max_retries(3));

        event.retry_count = 4;
        assert!(event.exceeded_max_retries(3));
    }

    #[test]
    fn test_serialization() {
        let payload = TestPayload {
            message: "test".to_string(),
        };
        let event = StreamEvent::new(Action::Create, payload);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"action\":\"create\""));
        assert!(json.contains("\"message\":\"test\""));

        let deserialized: StreamEvent<TestPayload> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.action, Action::Create);
        assert_eq!(deserialized.payload.message, "test");
    }
}
