//! Messaging abstraction layer
//!
//! Provides a trait-based abstraction for pub/sub messaging,
//! allowing easy switching between NATS and Dapr implementations.

mod nats_broker;

pub use nats_broker::NatsBroker;

use async_trait::async_trait;
use eyre::Result;
use serde::{de::DeserializeOwned, Serialize};

/// Event envelope containing metadata and payload
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct EventEnvelope<T> {
    /// Unique event ID
    pub id: String,
    /// Event type/name
    pub event_type: String,
    /// Source service
    pub source: String,
    /// Timestamp (RFC3339)
    pub timestamp: String,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    /// The actual event payload
    pub data: T,
}

impl<T> EventEnvelope<T> {
    pub fn new(event_type: impl Into<String>, source: impl Into<String>, data: T) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            source: source.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            correlation_id: None,
            data,
        }
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Received message with metadata
pub struct ReceivedMessage {
    /// Subject/topic the message was received on
    pub subject: String,
    /// Raw payload bytes
    pub payload: Vec<u8>,
    /// Reply subject for request-reply patterns
    pub reply: Option<String>,
}

impl ReceivedMessage {
    /// Deserialize payload to a typed event envelope
    pub fn parse<T: DeserializeOwned>(&self) -> Result<EventEnvelope<T>> {
        let envelope: EventEnvelope<T> = serde_json::from_slice(&self.payload)?;
        Ok(envelope)
    }

    /// Deserialize payload directly (without envelope)
    pub fn parse_payload<T: DeserializeOwned>(&self) -> Result<T> {
        let data: T = serde_json::from_slice(&self.payload)?;
        Ok(data)
    }
}

/// Abstract message broker interface
///
/// This trait allows swapping between different messaging backends:
/// - NATS (current implementation)
/// - Dapr pub/sub (future implementation)
/// - In-memory (for testing)
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Publish an event to a subject/topic
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        subject: &str,
        event: &EventEnvelope<T>,
    ) -> Result<()>;

    /// Publish raw bytes to a subject
    async fn publish_raw(&self, subject: &str, payload: &[u8]) -> Result<()>;

    /// Subscribe to a subject and receive messages
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn MessageStream>>;

    /// Request-reply pattern
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        subject: &str,
        request: &T,
    ) -> Result<R>;

    /// Create a queue group subscription (load-balanced across workers)
    async fn queue_subscribe(
        &self,
        subject: &str,
        queue_group: &str,
    ) -> Result<Box<dyn MessageStream>>;
}

/// Stream of incoming messages
#[async_trait]
pub trait MessageStream: Send + Sync {
    /// Receive the next message (blocks until available)
    async fn next(&mut self) -> Option<ReceivedMessage>;
}
