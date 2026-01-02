//! Products API - REST and gRPC server

use axum_helpers::server::{create_production_app, health_router};
use core_config::tracing::{init_tracing, install_color_eyre};
use domain_products::{MongoProductRepository, ProductService};
use rpc::products::products_service_server::ProductsServiceServer;
use std::net::SocketAddr;
use std::time::Duration;
use tonic::transport::Server as TonicServer;
use tracing::info;

mod api;
mod config;
mod grpc;
mod openapi;
mod state;

use config::Config;
use grpc::ProductsGrpcService;
use state::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    install_color_eyre();

    let config = Config::from_env()?;
    init_tracing(&config.environment);

    info!("Connecting to MongoDB at {}", config.mongodb.url());

    // Connect to MongoDB
    let mongo_client =
        database::mongodb::connect_from_config_with_retry(&config.mongodb, None).await?;

    let db = mongo_client.database(config.mongodb.database());

    info!(
        "Successfully connected to MongoDB database: {}",
        config.mongodb.database()
    );

    // Initialize the application state
    let state = AppState {
        config: config.clone(),
        mongo_client,
        db,
    };

    // Initialize indexes
    api::init_indexes(&state).await?;

    // Create a gRPC service
    let grpc_service = {
        let repository = MongoProductRepository::new(&state.db);
        let service = ProductService::new(repository);
        ProductsGrpcService::new(service)
    };

    // Start gRPC server in background
    let grpc_port = config.grpc_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;

    let grpc_handle = tokio::spawn(async move {
        info!("Starting gRPC server on {}", grpc_addr);
        TonicServer::builder()
            .add_service(ProductsServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .map_err(|e| eyre::eyre!("gRPC server error: {}", e))
    });

    // Build REST router
    let api_routes = api::routes(&state);
    let router = axum_helpers::create_router::<openapi::ApiDoc>(api_routes).await?;
    let app = router.merge(health_router(state.config.app.clone()));

    info!(
        "Starting Products API - REST on port {}, gRPC on port {}",
        state.config.server.port, grpc_port
    );

    // Run REST server with graceful shutdown
    let rest_result = create_production_app(
        app,
        &state.config.server,
        Duration::from_secs(30),
        async move {
            info!("Shutting down: closing MongoDB connections");
            drop(state.mongo_client);
            info!("MongoDB connection closed");
        },
    )
    .await;

    // Abort gRPC server on shutdown
    grpc_handle.abort();

    rest_result.map_err(|e| eyre::eyre!("Server error: {}", e))?;

    info!("Products API shutdown complete");
    Ok(())
}
