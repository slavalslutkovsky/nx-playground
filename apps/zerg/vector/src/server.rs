//! gRPC server initialization and lifecycle management
//!
//! This module handles all server setup:
//! - Tracing initialization
//! - Qdrant connection
//! - Optional embedding provider setup
//! - Service creation
//! - gRPC server configuration and startup
//! - Health check service (grpc.health.v1.Health)

use std::sync::Arc;

use core_config::Environment;
use domain_vector::{OpenAIProvider, QdrantConfig, QdrantRepository, VectorService};
use eyre::{Result, WrapErr};
use rpc::vector::vector_service_server::{SERVICE_NAME, VectorServiceServer};
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tracing::info;

use crate::service::VectorServiceImpl;

/// Run the gRPC server
///
/// This is the main entry point for server initialization. It:
/// 1. Sets up structured logging (env-aware: JSON for prod, pretty for dev)
/// 2. Connects to Qdrant vector database
/// 3. Optionally initializes embedding provider (OpenAI)
/// 4. Creates the repository and service layers
/// 5. Starts the gRPC server with compression enabled
///
/// # Errors
///
/// Returns an error if:
/// - Qdrant configuration is invalid
/// - Qdrant connection fails
/// - Server binding fails
/// - Server runtime encounters an error
pub async fn run() -> Result<()> {
    // Initialize tracing (env-aware: JSON for prod, pretty for dev)
    let environment = Environment::from_env();
    core_config::tracing::init_tracing(&environment);

    // Load Qdrant configuration from environment
    let qdrant_config = QdrantConfig::from_env().wrap_err("Failed to load Qdrant configuration")?;

    // Connect to Qdrant
    info!("Connecting to Qdrant at {}...", qdrant_config.url);
    let repository = QdrantRepository::new(qdrant_config)
        .await
        .wrap_err("Failed to connect to Qdrant")?;
    info!("Connected to Qdrant successfully");

    // Create a vector service
    let mut service = VectorService::new(repository);

    // Optionally configure embedding provider
    if let Ok(provider) = OpenAIProvider::from_env() {
        info!("OpenAI embedding provider configured");
        service = service.with_embedding_provider(Arc::new(provider));
    } else {
        info!("No OpenAI API key found, embedding operations will be disabled");
    }

    // Create gRPC service implementation
    let vector_service = VectorServiceImpl::new(service);

    // Configure server address from environment or default
    let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "[::1]".to_string());
    let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50052".to_string());
    let addr_str = format!("{}:{}", host, port);
    let addr = addr_str
        .parse()
        .wrap_err_with(|| format!("Failed to parse server address: {}", addr_str))?;
    info!("VectorService listening on {}", addr);
    info!("Using Zstd compression for optimal performance");

    // Create a health reporter for Kubernetes probes
    let (health_reporter, health_service) = health_reporter();

    // Mark vector service as serving (for k8s readiness/liveness probes)
    health_reporter
        .set_service_status(SERVICE_NAME, tonic_health::ServingStatus::Serving)
        .await;
    // Also set an empty service name for generic health checks (what k8s uses by default)
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;
    info!("Health check service enabled (grpc.health.v1.Health)");

    // Build and start the gRPC server with production settings
    Server::builder()
        .add_service(health_service)
        .add_service(
            VectorServiceServer::new(vector_service)
                // Enable zstd compression (3-5x faster than gzip, better compression ratio)
                .accept_compressed(tonic::codec::CompressionEncoding::Zstd)
                .send_compressed(tonic::codec::CompressionEncoding::Zstd),
        )
        .serve(addr)
        .await
        .wrap_err("gRPC server failed")?;

    Ok(())
}
