//! Stream registry types and definitions.
//!
//! This module provides:
//! - `StreamDef` trait for domain-specific stream definitions
//! - `Action` enum for standard CRUD actions
//! - `MessageKey` enum for standard message field keys
//! - `StreamId` enum for known streams (optional, for multi-stream workers)

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator};

/// Standard CRUD actions for stream events.
///
/// These actions represent the typical operations that can be performed
/// on domain entities via stream events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Create a new entity.
    Create,
    /// Read/get an entity.
    Read,
    /// Update an existing entity.
    Update,
    /// Delete an entity.
    Delete,
    /// List entities.
    List,
    /// Send/dispatch (for notifications, emails, etc.).
    Send,
    /// Process (generic action for background jobs).
    Process,
}

impl Action {
    /// Get all action variants.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

/// Standard message keys used in stream entries.
///
/// These are the common field names used when adding messages to Redis streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Display, AsRefStr, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MessageKey {
    /// The job/event payload (JSON serialized).
    Job,
    /// The action type.
    Action,
    /// Correlation ID for request tracing.
    RequestId,
    /// Retry count for failed jobs.
    RetryCount,
    /// Timestamp when the message was created.
    CreatedAt,
}

/// Stream definition trait.
///
/// Each domain implements this trait to define their stream configuration.
/// This enables type-safe stream configuration and consistent naming conventions.
///
/// # Example
///
/// ```rust,ignore
/// use stream_worker::StreamDef;
///
/// pub struct EmailStream;
///
/// impl StreamDef for EmailStream {
///     const STREAM_NAME: &'static str = "email:jobs";
///     const CONSUMER_GROUP: &'static str = "email_workers";
///     const DLQ_STREAM: &'static str = "email:dlq";
/// }
/// ```
pub trait StreamDef: Send + Sync {
    /// The Redis stream name (e.g., "email:jobs", "tasks:events").
    const STREAM_NAME: &'static str;

    /// The consumer group name for this stream.
    const CONSUMER_GROUP: &'static str;

    /// The dead letter queue stream name for failed jobs.
    const DLQ_STREAM: &'static str;

    /// Maximum stream length before auto-trim (MAXLEN).
    /// Default: 100,000 entries.
    const MAX_LENGTH: i64 = 100_000;

    /// Get the stream name.
    fn stream_name() -> &'static str {
        Self::STREAM_NAME
    }

    /// Get the consumer group name.
    fn consumer_group() -> &'static str {
        Self::CONSUMER_GROUP
    }

    /// Get the DLQ stream name.
    fn dlq_stream() -> &'static str {
        Self::DLQ_STREAM
    }
}

/// Known stream identifiers for the system.
///
/// This enum provides a central registry of all known streams,
/// useful for multi-stream workers or monitoring tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, AsRefStr, EnumString, EnumIter)]
pub enum StreamId {
    /// Email background jobs stream.
    #[strum(serialize = "email:jobs")]
    EmailJobs,
    /// Task domain events stream.
    #[strum(serialize = "tasks:events")]
    TaskEvents,
    /// Task commands stream (for CRUD operations).
    #[strum(serialize = "tasks:commands")]
    TaskCommands,
    /// User domain events stream.
    #[strum(serialize = "users:events")]
    UserEvents,
    /// Project domain events stream.
    #[strum(serialize = "projects:events")]
    ProjectEvents,
}

impl StreamId {
    /// Get the stream name string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::EmailJobs => "email:jobs",
            Self::TaskEvents => "tasks:events",
            Self::TaskCommands => "tasks:commands",
            Self::UserEvents => "users:events",
            Self::ProjectEvents => "projects:events",
        }
    }

    /// Get the consumer group for this stream.
    pub fn consumer_group(&self) -> &'static str {
        match self {
            Self::EmailJobs => "email_workers",
            Self::TaskEvents => "task_workers",
            Self::TaskCommands => "task_workers",
            Self::UserEvents => "user_workers",
            Self::ProjectEvents => "project_workers",
        }
    }

    /// Get the DLQ stream name.
    pub fn dlq(&self) -> String {
        let domain = self.name().split(':').next().unwrap_or("unknown");
        format!("{}:dlq", domain)
    }

    /// Get all stream names.
    pub fn all() -> Vec<&'static str> {
        Self::iter().map(|s| s.name()).collect()
    }

    /// Get all stream IDs.
    pub fn all_ids() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_serialization() {
        assert_eq!(Action::Create.to_string(), "create");
        assert_eq!(Action::Update.as_ref(), "update");
    }

    #[test]
    fn test_action_deserialization() {
        let action: Action = "create".parse().unwrap();
        assert_eq!(action, Action::Create);
    }

    #[test]
    fn test_message_key() {
        assert_eq!(MessageKey::Job.to_string(), "job");
        assert_eq!(MessageKey::RequestId.as_ref(), "request_id");
    }

    #[test]
    fn test_stream_id() {
        assert_eq!(StreamId::EmailJobs.name(), "email:jobs");
        assert_eq!(StreamId::EmailJobs.consumer_group(), "email_workers");
        assert_eq!(StreamId::EmailJobs.dlq(), "email:dlq");
    }

    #[test]
    fn test_stream_id_all() {
        let all = StreamId::all();
        assert!(all.contains(&"email:jobs"));
        assert!(all.contains(&"tasks:events"));
    }

    struct TestStream;
    impl StreamDef for TestStream {
        const STREAM_NAME: &'static str = "test:stream";
        const CONSUMER_GROUP: &'static str = "test_workers";
        const DLQ_STREAM: &'static str = "test:dlq";
    }

    #[test]
    fn test_stream_def() {
        assert_eq!(TestStream::stream_name(), "test:stream");
        assert_eq!(TestStream::consumer_group(), "test_workers");
        assert_eq!(TestStream::dlq_stream(), "test:dlq");
        assert_eq!(TestStream::MAX_LENGTH, 100_000);
    }
}
