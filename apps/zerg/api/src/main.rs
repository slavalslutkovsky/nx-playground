use axum_helpers::server::{create_production_app, health_router};
use core_config::tracing::{init_tracing, install_color_eyre};
use std::time::Duration;
use tracing::info;

mod api;
mod config;
mod events;
mod grpc_pool;
mod openapi;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Install color-eyre first for colored error output (before any fallible operations)
    install_color_eyre();

    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing with ErrorLayer for span trace capture
    init_tracing(&config.environment);

    let tasks_addr =
        std::env::var("TASKS_SERVICE_ADDR").unwrap_or_else(|_| "http://[::1]:50051".to_string());

    info!("Connecting to TasksService at {} (optimized)", tasks_addr);

    let tasks_client = grpc_pool::create_optimized_tasks_client(tasks_addr).await?;

    // Initialize database connections concurrently
    let postgres_future = async {
        database::postgres::connect_from_config_with_retry(config.database.clone(), None)
            .await
            .map_err(|e| eyre::eyre!("PostgreSQL connection failed: {}", e))
    };

    let redis_future = async {
        database::redis::connect_from_config_with_retry(config.redis.clone(), None)
            .await
            .map_err(|e| eyre::eyre!("Redis connection failed: {}", e))
    };

    let (db, redis) = tokio::try_join!(postgres_future, redis_future)?;

    // Initialize JWT + Redis authentication
    let jwt_auth = axum_helpers::JwtRedisAuth::new(redis.clone(), &config.jwt)
        .map_err(|e| eyre::eyre!("Failed to initialize JWT auth: {}", e))?;

    // Initialize optional vector/graph database clients
    let qdrant = match std::env::var("QDRANT_URL") {
        Ok(url) => {
            info!("Connecting to Qdrant at {}", url);
            match state::QdrantState::new(&url).await {
                Ok(client) => {
                    info!("Qdrant connected successfully");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to Qdrant: {}", e);
                    None
                }
            }
        }
        Err(_) => None,
    };

    let neo4j = match (
        std::env::var("NEO4J_URI"),
        std::env::var("NEO4J_USER"),
        std::env::var("NEO4J_PASSWORD"),
    ) {
        (Ok(uri), Ok(user), Ok(password)) => {
            info!("Connecting to Neo4j at {}", uri);
            match state::Neo4jState::new(&uri, &user, &password).await {
                Ok(client) => {
                    info!("Neo4j connected successfully");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to Neo4j: {}", e);
                    None
                }
            }
        }
        _ => None,
    };

    let arangodb = match (
        std::env::var("ARANGO_URL"),
        std::env::var("ARANGO_USER"),
        std::env::var("ARANGO_PASSWORD"),
        std::env::var("ARANGO_DATABASE"),
    ) {
        (Ok(url), Ok(user), Ok(password), Ok(database)) => {
            info!("Connecting to ArangoDB at {}", url);
            match state::ArangoState::new(&url, &user, &password, &database).await {
                Ok(client) => {
                    info!("ArangoDB connected successfully");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to ArangoDB: {}", e);
                    None
                }
            }
        }
        _ => None,
    };

    let milvus = match std::env::var("MILVUS_URL") {
        Ok(url) => {
            info!("Configuring Milvus client for {}", url);
            Some(state::MilvusState::new(&url))
        }
        Err(_) => None,
    };

    // Initialize NATS event publisher (optional)
    let events = match std::env::var("NATS_URL") {
        Ok(url) => {
            info!("Connecting to NATS at {}", url);
            match async_nats::connect(&url).await {
                Ok(client) => {
                    info!("NATS connected successfully");
                    Some(events::EventPublisher::new(client))
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to NATS: {}", e);
                    None
                }
            }
        }
        Err(_) => None,
    };

    // Initialize the application state with database connections
    let state = AppState {
        config,
        tasks_client,
        db,
        redis,
        jwt_auth,
        qdrant,
        neo4j,
        arangodb,
        milvus,
        events,
    };

    // Build router with API routes (pass reference, not ownership!)
    let api_routes = api::routes(&state);

    // create_router adds docs/middleware to our composed routes
    let router = axum_helpers::create_router::<openapi::ApiDoc>(api_routes).await?;

    // Merge health endpoints into the app
    // - /health: liveness check with app name/version
    // - /ready: readiness check with actual db/redis health checks
    let app = router
        .merge(health_router(state.config.app.clone()))
        .merge(api::ready_router(state.clone()));

    info!("Starting zerg API with production-ready shutdown (30s timeout)");

    // Production-ready server with graceful shutdown and cleanup
    // State moves here for cleanup
    create_production_app(
        app,
        &state.config.server,
        Duration::from_secs(30), // 30s graceful shutdown timeout
        async move {
            info!("Shutting down: closing database connections");

            // Close connections concurrently
            tokio::join!(
                async {
                    match state.db.close().await {
                        Ok(_) => info!("PostgreSQL connection closed successfully"),
                        Err(e) => tracing::error!("Error closing PostgreSQL: {}", e),
                    }
                },
                async {
                    // Redis ConnectionManager closes automatically on drop
                    drop(state.redis);
                    info!("Redis connection closed successfully");
                }
            );
        },
    )
    .await
    .map_err(|e| eyre::eyre!("Server error: {}", e))?;

    info!("Zerg API shutdown complete");
    Ok(())
}
