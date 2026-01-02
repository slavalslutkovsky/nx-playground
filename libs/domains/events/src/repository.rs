//! Event repository trait

use crate::error::Result;
use crate::models::{Event, EventFilter, EventStats};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for event storage operations
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Store a new event
    async fn create(&self, event: Event) -> Result<Event>;

    /// Store multiple events in batch
    async fn create_batch(&self, events: Vec<Event>) -> Result<Vec<Event>>;

    /// Get event by ID
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<Event>>;

    /// List events with filtering
    async fn list(&self, filter: &EventFilter) -> Result<Vec<Event>>;

    /// Count events matching filter
    async fn count(&self, filter: &EventFilter) -> Result<u64>;

    /// Get event statistics
    async fn stats(&self) -> Result<EventStats>;

    /// Delete event by ID
    async fn delete(&self, id: &Uuid) -> Result<bool>;

    /// Delete events older than a certain date
    async fn delete_before(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64>;
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::mock;

    mock! {
        pub EventRepository {}

        #[async_trait]
        impl EventRepository for EventRepository {
            async fn create(&self, event: Event) -> Result<Event>;
            async fn create_batch(&self, events: Vec<Event>) -> Result<Vec<Event>>;
            async fn get_by_id(&self, id: &Uuid) -> Result<Option<Event>>;
            async fn list(&self, filter: &EventFilter) -> Result<Vec<Event>>;
            async fn count(&self, filter: &EventFilter) -> Result<u64>;
            async fn stats(&self) -> Result<EventStats>;
            async fn delete(&self, id: &Uuid) -> Result<bool>;
            async fn delete_before(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64>;
        }
    }
}
