//! Zerg NATS Worker
//!
//! Event-driven worker service that communicates via NATS messaging.
//! Designed with a messaging abstraction layer for future Dapr compatibility.

mod config;
mod handlers;
mod messaging;

use crate::config::Config;
use crate::handlers::TaskEventHandler;
use crate::messaging::{MessageBroker, NatsBroker};
use eyre::Result;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing();

    info!("Starting Zerg NATS Worker");

    // Load configuration
    let config = Config::from_env();

    // Create a messaging broker (NATS implementation, swappable to Dapr later)
    let broker = NatsBroker::connect(&config.nats_url).await?;
    let broker = Arc::new(broker);

    info!(nats_url = %config.nats_url, "Connected to NATS");

    // Create an event handler
    let handler = TaskEventHandler::new(broker.clone());

    // Subscribe to topics
    let subjects = vec![
        "tasks.created",
        "tasks.updated",
        "tasks.deleted",
        "tasks.status_changed",
    ];

    for subject in &subjects {
        info!(subject = %subject, "Subscribing to subject");
    }

    // Run the worker
    handler.run(&subjects).await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let is_prod = std::env::var("ENVIRONMENT")
        .map(|e| e == "production")
        .unwrap_or(false);

    if is_prod {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().pretty())
            .init();
    }
}
