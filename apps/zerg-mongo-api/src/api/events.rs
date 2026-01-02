//! Events API routes
//!
//! Integrates the events domain with MongoDB, InfluxDB, and Dapr pub/sub.

use crate::state::AppState;
use axum::Router;
use domain_events::{
    DaprClient, DaprEventPublisher, EventService, InfluxEventStore, MongoEventRepository,
};
use std::sync::Arc;
use tracing::info;

/// Create the events router with full integration
pub fn router(state: &AppState) -> Router {
    // Create a MongoDB repository
    let repository = MongoEventRepository::new(&state.db);

    // Create InfluxDB store (optional, from env)
    let influx = InfluxEventStore::from_env();

    // Create a Dapr client and publisher
    let dapr = DaprClient::from_env();
    let publisher = DaprEventPublisher::new(dapr, "events", "zerg-mongo-api");

    // Create an event service with all integrations
    let mut service = EventService::new(repository);

    if let Some(influx_store) = influx {
        info!("InfluxDB integration enabled for events");
        service = service.with_influx(influx_store);
    }

    service = service.with_publisher(publisher);

    let service_state = Arc::new(service);

    // Use the domain's router
    domain_events::events_router().with_state(service_state)
}

/// Initialize event indexes in MongoDB
pub async fn init_indexes(db: &mongodb::Database) -> eyre::Result<()> {
    let repository = MongoEventRepository::new(db);
    repository
        .create_indexes()
        .await
        .map_err(|e| eyre::eyre!("Failed to create event indexes: {}", e))?;
    info!("Event collection indexes created");
    Ok(())
}
