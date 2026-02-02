//! gRPC server initialization and lifecycle management
//!
//! This module handles all server setup:
//! - Tracing initialization
//! - Database connection
//! - Service creation
//! - gRPC server configuration and startup
//! - Health check service (grpc.health.v1.Health)

use core_config::{Environment, FromEnv};
use database::postgres::PostgresConfig;
use domain_tasks::{PgTaskRepository, TaskService};
use eyre::{Result, WrapErr};
use grpc_client::server::{GrpcServer, ServerConfig, create_health_service};
use rpc::tasks::tasks_service_server::{SERVICE_NAME, TasksServiceServer};
use tonic::codec::CompressionEncoding;
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
    let db_config = PostgresConfig::from_env().wrap_err("Failed to load database configuration")?;

    // Load gRPC server configuration
    let server_config = ServerConfig::from_env().wrap_err("Failed to load server configuration")?;

    // Connect to a database with retry logic
    info!("Connecting to database...");
    let db = database::postgres::connect_from_config_with_retry(db_config, None)
        .await
        .wrap_err("Failed to connect to database")?;
    info!("Connected to database successfully");

    // Create repository and service layers
    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);

    // Create gRPC service implementation with compression
    let tasks_service = TasksServiceServer::new(TasksServiceImpl::new(service))
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd);

    // Create health service
    let (health_reporter, health_service) = create_health_service();

    // Mark service as serving (for k8s readiness/liveness probes)
    GrpcServer::setup_health(&health_reporter, SERVICE_NAME).await;

    // Log startup
    GrpcServer::log_startup(&server_config, SERVICE_NAME);

    // Get socket address
    let addr = server_config
        .socket_addr()
        .wrap_err("Invalid server address")?;

    // Build and start the gRPC server
    Server::builder()
        .add_service(health_service)
        .add_service(tasks_service)
        .serve(addr)
        .await
        .wrap_err("gRPC server failed")?;

    Ok(())
}
