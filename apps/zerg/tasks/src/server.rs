//! gRPC server initialization and lifecycle management
//!
//! This module handles all server setup:
//! - Tracing initialization
//! - Database connection
//! - Service creation
//! - gRPC server configuration and startup

use core_config::{Environment, FromEnv};
use database::postgres::PostgresConfig;
use domain_tasks::{PgTaskRepository, TaskService};
use eyre::{Result, WrapErr};
use rpc::tasks::tasks_service_server::TasksServiceServer;
use tonic::transport::Server;
use tracing::info;

use crate::service::TasksServiceImpl;

/// Run the gRPC server
///
/// This is the main entry point for server initialization. It:
/// 1. Sets up structured logging (env-aware: JSON for prod, pretty for dev)
/// 2. Connects to the database with retry logic
/// 3. Creates the repository and service layers
/// 4. Starts the gRPC server with compression enabled
///
/// # Errors
///
/// Returns an error if:
/// - Database configuration is invalid
/// - Database connection fails
/// - Server binding fails
/// - Server runtime encounters an error
pub async fn run() -> Result<()> {
    // Initialize tracing (env-aware: JSON for prod, pretty for dev)
    let environment = Environment::from_env();
    core_config::tracing::init_tracing(&environment);

    // Load database configuration from environment
    let config = PostgresConfig::from_env().wrap_err("Failed to load database configuration")?;

    // Connect to a database with retry logic
    info!("Connecting to database...");
    let db = database::postgres::connect_from_config_with_retry(config, None)
        .await
        .wrap_err("Failed to connect to database")?;
    info!("Connected to database successfully");

    // Create repository and service layers
    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);

    // Create gRPC service implementation
    let tasks_service = TasksServiceImpl::new(service);

    // Configure server address
    let addr = "[::1]:50051"
        .parse()
        .wrap_err("Failed to parse server address")?;
    info!("TasksService listening on {}", addr);
    info!("Using Zstd compression for optimal performance");

    // Build and start the gRPC server with production settings
    Server::builder()
        .add_service(
            TasksServiceServer::new(tasks_service)
                // Enable zstd compression (3-5x faster than gzip, better compression ratio)
                .accept_compressed(tonic::codec::CompressionEncoding::Zstd)
                .send_compressed(tonic::codec::CompressionEncoding::Zstd),
        )
        .serve(addr)
        .await
        .wrap_err("gRPC server failed")?;

    Ok(())
}
