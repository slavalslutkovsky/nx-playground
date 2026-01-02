//! Event domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Event severity levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for EventSeverity {
    fn default() -> Self {
        Self::Info
    }
}

/// Event categories for classification
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum EventCategory {
    /// System-level events (startup, shutdown, health)
    System,
    /// User actions and interactions
    User,
    /// API requests and responses
    Api,
    /// Business logic events
    Business,
    /// Security-related events
    Security,
    /// Performance metrics
    Performance,
    /// Integration events (external services)
    Integration,
    /// Custom application events
    Custom,
}

impl Default for EventCategory {
    fn default() -> Self {
        Self::Custom
    }
}

/// Event metadata for additional context
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct EventMetadata {
    /// Source service/application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Correlation ID for distributed tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// User ID if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Session ID if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// IP address of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Additional custom fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Main Event entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Event {
    /// Unique identifier
    #[serde(rename = "_id", alias = "id")]
    pub id: Uuid,

    /// Event name/type
    pub name: String,

    /// Event category
    pub category: EventCategory,

    /// Severity level
    pub severity: EventSeverity,

    /// Human-readable message
    pub message: String,

    /// Event payload data
    #[serde(default)]
    pub data: serde_json::Value,

    /// Event metadata
    #[serde(default)]
    pub metadata: EventMetadata,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,

    /// When the event was stored
    pub created_at: DateTime<Utc>,
}

impl Event {
    /// Create a new event with auto-generated ID and timestamps
    pub fn new(
        name: impl Into<String>,
        category: EventCategory,
        severity: EventSeverity,
        message: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            category,
            severity,
            message: message.into(),
            data: serde_json::Value::Null,
            metadata: EventMetadata::default(),
            tags: Vec::new(),
            timestamp: now,
            created_at: now,
        }
    }

    /// Builder: set event data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Builder: set metadata
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Builder: set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder: set custom timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Builder: set source in metadata
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.metadata.source = Some(source.into());
        self
    }

    /// Builder: set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(correlation_id.into());
        self
    }
}

/// DTO for creating new events
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateEvent {
    /// Event name/type
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,

    /// Event category
    #[serde(default)]
    pub category: EventCategory,

    /// Severity level
    #[serde(default)]
    pub severity: EventSeverity,

    /// Human-readable message
    #[validate(length(min = 1, max = 10000, message = "Message must be 1-10000 characters"))]
    pub message: String,

    /// Event payload data
    #[serde(default)]
    pub data: Option<serde_json::Value>,

    /// Event metadata
    #[serde(default)]
    pub metadata: Option<EventMetadata>,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom timestamp (defaults to now if not provided)
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

impl From<CreateEvent> for Event {
    fn from(create: CreateEvent) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: create.name,
            category: create.category,
            severity: create.severity,
            message: create.message,
            data: create.data.unwrap_or(serde_json::Value::Null),
            metadata: create.metadata.unwrap_or_default(),
            tags: create.tags,
            timestamp: create.timestamp.unwrap_or(now),
            created_at: now,
        }
    }
}

/// Filter options for querying events
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct EventFilter {
    /// Filter by category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<EventCategory>,

    /// Filter by severity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<EventSeverity>,

    /// Filter by minimum severity (includes this and higher)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_severity: Option<EventSeverity>,

    /// Filter by event name (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Filter by source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Filter by correlation ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Filter by tags (any match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Filter by start time (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<DateTime<Utc>>,

    /// Filter by end time (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<DateTime<Utc>>,

    /// Full-text search in message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Pagination: offset
    #[serde(default)]
    pub offset: u64,

    /// Pagination: limit
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    50
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventStats {
    /// Total event count
    pub total: u64,

    /// Count by category
    pub by_category: HashMap<String, u64>,

    /// Count by severity
    pub by_severity: HashMap<String, u64>,

    /// Events in the last hour
    pub last_hour: u64,

    /// Events in the last 24 hours
    pub last_24h: u64,
}

/// Dapr CloudEvent envelope for pub/sub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent<T> {
    /// CloudEvent spec version
    pub specversion: String,

    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,

    /// Event source
    pub source: String,

    /// Unique event ID
    pub id: String,

    /// Timestamp
    pub time: DateTime<Utc>,

    /// Content type
    pub datacontenttype: String,

    /// Event data
    pub data: T,
}

impl<T: Serialize> CloudEvent<T> {
    /// Create a new CloudEvent wrapper
    pub fn new(event_type: impl Into<String>, source: impl Into<String>, data: T) -> Self {
        Self {
            specversion: "1.0".to_string(),
            event_type: event_type.into(),
            source: source.into(),
            id: Uuid::now_v7().to_string(),
            time: Utc::now(),
            datacontenttype: "application/json".to_string(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            "user.login",
            EventCategory::User,
            EventSeverity::Info,
            "User logged in",
        )
        .with_source("auth-service")
        .with_tags(vec!["auth".to_string(), "login".to_string()]);

        assert!(!event.id.is_nil());
        assert_eq!(event.name, "user.login");
        assert_eq!(event.category, EventCategory::User);
        assert_eq!(event.metadata.source, Some("auth-service".to_string()));
        assert_eq!(event.tags.len(), 2);
    }

    #[test]
    fn test_create_event_to_event() {
        let create = CreateEvent {
            name: "test.event".to_string(),
            category: EventCategory::Api,
            severity: EventSeverity::Warning,
            message: "Test message".to_string(),
            data: Some(serde_json::json!({"key": "value"})),
            metadata: None,
            tags: vec!["test".to_string()],
            timestamp: None,
        };

        let event: Event = create.into();
        assert_eq!(event.name, "test.event");
        assert_eq!(event.category, EventCategory::Api);
    }
}
