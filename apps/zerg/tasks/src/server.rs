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
use rpc::tasks::tasks_service_server::TasksServiceServer;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
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

    // Configure server address from environment or default
    let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "[::1]".to_string());
    let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr_str = format!("{}:{}", host, port);
    let addr = addr_str
        .parse()
        .wrap_err_with(|| format!("Failed to parse server address: {}", addr_str))?;
    info!("TasksService listening on {}", addr);
    info!("Using Zstd compression for optimal performance");

    // Create health reporter for Kubernetes probes
    let (health_reporter, health_service) = health_reporter();

    // Mark tasks service as serving (for k8s readiness/liveness probes)
    // Using the service name from the proto definition
    health_reporter
        .set_service_status("tasks.TasksService", tonic_health::ServingStatus::Serving)
        .await;
    // Also set empty service name for generic health checks (what k8s uses by default)
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;
    info!("Health check service enabled (grpc.health.v1.Health)");

    // Build and start the gRPC server with production settings
    Server::builder()
        .add_service(health_service)
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
