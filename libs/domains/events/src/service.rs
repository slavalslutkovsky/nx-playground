//! Event service layer

use crate::dapr::DaprEventPublisher;
use crate::error::{EventError, Result};
use crate::influxdb::InfluxEventStore;
use crate::models::{CreateEvent, Event, EventFilter, EventStats};
use crate::repository::EventRepository;
use std::sync::Arc;
use tracing::{info, instrument, warn};
use validator::Validate;

/// Event service that coordinates MongoDB, InfluxDB, and Dapr
pub struct EventService<R: EventRepository> {
    repository: R,
    influx: Option<Arc<InfluxEventStore>>,
    publisher: Option<Arc<DaprEventPublisher>>,
}

impl<R: EventRepository> EventService<R> {
    /// Create a new event service with just MongoDB
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            influx: None,
            publisher: None,
        }
    }

    /// Add InfluxDB integration
    pub fn with_influx(mut self, influx: InfluxEventStore) -> Self {
        self.influx = Some(Arc::new(influx));
        self
    }

    /// Add Dapr publisher integration
    pub fn with_publisher(mut self, publisher: DaprEventPublisher) -> Self {
        self.publisher = Some(Arc::new(publisher));
        self
    }

    /// Create and store a new event
    #[instrument(skip(self, create), fields(event_name = %create.name))]
    pub async fn create(&self, create: CreateEvent) -> Result<Event> {
        // Validate input
        create.validate()?;

        // Convert to event
        let event: Event = create.into();

        // Store in MongoDB
        let event = self.repository.create(event).await?;
        info!(event_id = %event.id, "Event stored in MongoDB");

        // Write to InfluxDB (non-blocking, log errors)
        if let Some(influx) = &self.influx {
            let influx = Arc::clone(influx);
            let event_clone = event.clone();
            tokio::spawn(async move {
                if let Err(e) = influx.write_event(&event_clone).await {
                    warn!(error = %e, "Failed to write event to InfluxDB");
                }
            });
        }

        // Publish to Dapr (non-blocking, log errors)
        if let Some(publisher) = &self.publisher {
            let publisher = Arc::clone(publisher);
            let event_clone = event.clone();
            tokio::spawn(async move {
                if let Err(e) = publisher.publish(&event_clone).await {
                    warn!(error = %e, "Failed to publish event to Dapr");
                }
            });
        }

        Ok(event)
    }

    /// Create multiple events in batch
    #[instrument(skip(self, creates), fields(count = creates.len()))]
    pub async fn create_batch(&self, creates: Vec<CreateEvent>) -> Result<Vec<Event>> {
        // Validate all inputs
        for create in &creates {
            create.validate()?;
        }

        // Convert to events
        let events: Vec<Event> = creates.into_iter().map(Into::into).collect();

        // Store in MongoDB
        let events = self.repository.create_batch(events).await?;
        info!(count = events.len(), "Events batch stored in MongoDB");

        // Write to InfluxDB in batch
        if let Some(influx) = &self.influx {
            let influx = Arc::clone(influx);
            let events_clone = events.clone();
            tokio::spawn(async move {
                if let Err(e) = influx.write_events(&events_clone).await {
                    warn!(error = %e, "Failed to write events batch to InfluxDB");
                }
            });
        }

        // Publish each event to Dapr
        if let Some(publisher) = &self.publisher {
            for event in &events {
                let publisher = Arc::clone(publisher);
                let event_clone = event.clone();
                tokio::spawn(async move {
                    if let Err(e) = publisher.publish(&event_clone).await {
                        warn!(error = %e, event_id = %event_clone.id, "Failed to publish event");
                    }
                });
            }
        }

        Ok(events)
    }

    /// Get event by ID
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &uuid::Uuid) -> Result<Event> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| EventError::NotFound { id: id.to_string() })
    }

    /// List events with filtering
    #[instrument(skip(self, filter))]
    pub async fn list(&self, filter: &EventFilter) -> Result<Vec<Event>> {
        self.repository.list(filter).await
    }

    /// Count events matching filter
    #[instrument(skip(self, filter))]
    pub async fn count(&self, filter: &EventFilter) -> Result<u64> {
        self.repository.count(filter).await
    }

    /// Get event statistics
    #[instrument(skip(self))]
    pub async fn stats(&self) -> Result<EventStats> {
        self.repository.stats().await
    }

    /// Delete event by ID
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &uuid::Uuid) -> Result<bool> {
        self.repository.delete(id).await
    }

    /// Delete events older than a certain date
    #[instrument(skip(self))]
    pub async fn cleanup(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let deleted = self.repository.delete_before(before).await?;
        info!(deleted_count = deleted, "Old events cleaned up");
        Ok(deleted)
    }

    /// Query InfluxDB for time-series analytics
    #[instrument(skip(self))]
    pub async fn query_metrics(&self, flux_query: &str) -> Result<String> {
        let influx = self.influx.as_ref().ok_or_else(|| EventError::Internal {
            message: "InfluxDB not configured".to_string(),
        })?;

        influx.query(flux_query).await
    }

    /// Health check for all backends
    pub async fn health(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus::default();

        // Check MongoDB (via a simple count)
        status.mongodb = self.repository.count(&EventFilter::default()).await.is_ok();

        // Check InfluxDB
        if let Some(influx) = &self.influx {
            status.influxdb = influx.health().await.unwrap_or(false);
        }

        // Check Dapr
        if let Some(publisher) = &self.publisher {
            status.dapr = publisher.dapr.health().await.unwrap_or(false);
        }

        Ok(status)
    }
}

/// Health status for event service backends
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HealthStatus {
    pub mongodb: bool,
    pub influxdb: bool,
    pub dapr: bool,
}
