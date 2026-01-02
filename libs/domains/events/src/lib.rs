//! Events Domain
//!
//! Handles event ingestion, storage, and querying with:
//! - MongoDB for event persistence (full event data)
//! - InfluxDB for time-series metrics and analytics
//! - Dapr pub/sub for event distribution
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Event Flow                               │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  HTTP POST /events ─┬─► MongoDB (full event storage)        │
//! │                     │                                        │
//! │                     ├─► InfluxDB (metrics/time-series)      │
//! │                     │                                        │
//! │                     └─► Dapr Pub/Sub (event distribution)   │
//! │                              │                               │
//! │                              ▼                               │
//! │                     Other Microservices                      │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use utoipa::OpenApi;

mod dapr;
mod error;
mod handlers;
mod influxdb;
mod models;
mod mongodb;
mod repository;
mod service;

pub use dapr::{DaprClient, DaprEventPublisher};
pub use error::{EventError, Result};
pub use handlers::{CountResponse, events_router};
pub use influxdb::{EventMetrics, InfluxEventStore};
pub use models::{
    CreateEvent, Event, EventCategory, EventFilter, EventMetadata, EventSeverity, EventStats,
};
pub use mongodb::MongoEventRepository;
pub use repository::EventRepository;
pub use service::{EventService, HealthStatus};

/// OpenAPI documentation for Events API
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::list_events,
        handlers::create_event,
        handlers::create_events_batch,
        handlers::get_event,
        handlers::delete_event,
        handlers::count_events,
        handlers::get_stats,
        handlers::health_check,
    ),
    components(schemas(
        Event,
        CreateEvent,
        EventCategory,
        EventSeverity,
        EventMetadata,
        EventFilter,
        EventStats,
        CountResponse,
        HealthStatus,
    )),
    tags(
        (name = "events", description = "Event management with MongoDB, InfluxDB, and Dapr")
    )
)]
pub struct ApiDoc;
