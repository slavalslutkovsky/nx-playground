use axum_helpers::server::{create_production_app, health_router};
use core_config::tracing::{init_tracing, install_color_eyre};
use std::time::Duration;
use tracing::info;

mod api;
mod config;
mod openapi;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Install color-eyre first for colored error output
    install_color_eyre();

    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing
    init_tracing(&config.environment);

    info!("Connecting to MongoDB at {}", config.mongodb.url());

    // Connect to MongoDB with retry
    let mongo_client =
        database::mongodb::connect_from_config_with_retry(&config.mongodb, None).await?;

    // Get the database
    let db = mongo_client.database(config.mongodb.database());

    info!(
        "Successfully connected to MongoDB database: {}",
        config.mongodb.database()
    );

    // Initialize indexes
    api::events::init_indexes(&db).await?;

    // Initialize the application state
    let state = AppState {
        config,
        mongo_client,
        db,
    };

    // Build router with API routes
    let api_routes = api::routes(&state);

    // Create a router with OpenAPI docs
    let router = axum_helpers::create_router::<openapi::ApiDoc>(api_routes).await?;

    // Merge health endpoints
    let app = router.merge(health_router(state.config.app.clone()));

    info!("Starting Zerg MongoDB API with production-ready shutdown (30s timeout)");

    // Production-ready server with graceful shutdown
    create_production_app(
        app,
        &state.config.server,
        Duration::from_secs(30),
        async move {
            info!("Shutting down: closing MongoDB connections");
            // MongoDB client closes automatically on drop
            drop(state.mongo_client);
            info!("MongoDB connection closed successfully");
        },
    )
    .await
    .map_err(|e| eyre::eyre!("Server error: {}", e))?;

    info!("Zerg MongoDB API shutdown complete");
    Ok(())
}
