//! Tasks Worker Service
//!
//! A background worker that processes task commands from a Redis stream.
//!
//! ## Architecture
//!
//! ```text
//! Redis Stream (tasks:commands)
//!   ↓ (Consumer Group: task_workers)
//! StreamWorker<TaskCommand, TaskProcessor>
//!   ↓ (executes CRUD operations)
//! TaskService<PgTaskRepository>
//!   ↓
//! PostgreSQL Database
//!   ↓
//! Redis Stream (tasks:results)
//! ```
//!
//! ## Features
//!
//! - Consumer group support for horizontal scaling
//! - Automatic retry with exponential backoff
//! - Dead letter queue for failed commands
//! - Graceful shutdown handling
//! - Health check endpoint for Kubernetes probes
//! - Request/Response pattern with correlation IDs

use axum::Router;
use core_config::{app_info, Environment, FromEnv};
use database::{
    postgres::{connect_from_config_with_retry, PostgresConfig},
    redis::RedisConfig,
};
use domain_tasks::{PgTaskRepository, TaskCommand, TaskCommandStream, TaskProcessor, TaskService};
use eyre::{Result, WrapErr};
use std::sync::Arc;
use stream_worker::{full_admin_router, metrics, HealthState, StreamWorker, WorkerConfig};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::watch;
use tracing::{error, info};

/// Start the health and admin HTTP server
///
/// Provides endpoints for:
/// - Liveness probes: `/health`, `/healthz`
/// - Readiness probes: `/ready`, `/readyz`
/// - Stream monitoring: `/stream/info`
/// - Prometheus metrics: `/metrics`
/// - DLQ admin: `/admin/dlq/*`
async fn start_health_server(health_state: HealthState, port: u16) -> Result<()> {
    // Use full_admin_router which includes both health and DLQ admin endpoints
    let app: Router = full_admin_router(health_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .wrap_err_with(|| format!("Failed to bind health server to {}", addr))?;

    info!(port = %port, "Health and admin server listening");

    axum::serve(listener, app)
        .await
        .wrap_err("Health server failed")?;

    Ok(())
}

/// Run the tasks worker
///
/// This is the main entry point for the worker. It:
/// 1. Sets up structured logging (env-aware: JSON for prod, pretty for dev)
/// 2. Connects to PostgreSQL for task persistence
/// 3. Connects to Redis for stream processing
/// 4. Starts the worker with graceful shutdown handling
///
/// # Errors
///
/// Returns an error if:
/// - PostgreSQL configuration is invalid
/// - PostgreSQL connection fails
/// - Redis configuration is invalid
/// - Redis connection fails
/// - Worker encounters a fatal error
pub async fn run() -> Result<()> {
    // Initialize tracing (env-aware: JSON for prod, pretty for dev)
    let environment = Environment::from_env();
    core_config::tracing::init_tracing(&environment);

    // Initialize Prometheus metrics
    metrics::init_metrics();

    // App info for health endpoint
    let app_info = app_info!();

    info!(name = %app_info.name, version = %app_info.version, "Starting tasks worker service");
    info!("Environment: {:?}", environment);

    // Health server port (default 8082)
    // Checks TASKS_WORKER_HEALTH_PORT first, then HEALTH_PORT, then default
    let health_port: u16 = std::env::var("TASKS_WORKER_HEALTH_PORT")
        .or_else(|_| std::env::var("HEALTH_PORT"))
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .unwrap_or(8082);

    // Load PostgreSQL configuration from environment
    let pg_config =
        PostgresConfig::from_env().wrap_err("Failed to load PostgreSQL configuration")?;

    // Connect to PostgreSQL with retry logic
    info!("Connecting to PostgreSQL...");
    let db = connect_from_config_with_retry(pg_config, None)
        .await
        .wrap_err("Failed to connect to PostgreSQL")?;
    info!("Connected to PostgreSQL successfully");

    // Load Redis configuration from the environment
    let redis_config = RedisConfig::from_env().wrap_err("Failed to load Redis configuration")?;

    // Connect to Redis with retry logic
    info!("Connecting to Redis...");
    let redis = database::redis::connect_from_config_with_retry(redis_config, None)
        .await
        .wrap_err("Failed to connect to Redis")?;
    info!("Connected to Redis successfully");

    // Create worker configuration from TaskCommandStream definition
    // Production-ready settings: blocking mode + concurrency
    // Note: max_concurrent_jobs should be less than DB pool to avoid exhaustion
    let worker_config = WorkerConfig::from_stream_def::<TaskCommandStream>()
        .with_blocking(1000)            // Block for 1 second (instant delivery + clean shutdown)
        .with_batch_size(50)            // Process up to 50 jobs per read
        .with_max_concurrent_jobs(20);  // Process 20 jobs in parallel (conservative for shared DB)
    info!(
        stream = %worker_config.stream_name,
        consumer_group = %worker_config.consumer_group,
        consumer_id = %worker_config.consumer_id,
        block_timeout_ms = ?worker_config.block_timeout_ms,
        batch_size = %worker_config.batch_size,
        max_concurrent_jobs = %worker_config.max_concurrent_jobs,
        "Worker configuration loaded"
    );

    // Create the task repository and service
    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);

    // Create the processor with a result producer for async responses
    let processor = TaskProcessor::with_result_producer(service, redis.clone());
    info!("Task processor initialized");

    // Set up a shutdown signal
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn shutdown signal handler
    tokio::spawn(async move {
        if let Err(e) = shutdown_signal().await {
            error!("Error waiting for shutdown signal: {}", e);
        }
        let _ = shutdown_tx.send(true);
    });

    // Create a health state
    let health_state = HealthState::new(
        Arc::new(redis.clone()),
        app_info.name,
        app_info.version,
        worker_config.stream_name.clone(),
    );

    // Start health server in background
    let health_state_clone = health_state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_state_clone, health_port).await {
            error!(error = %e, "Health server failed");
        }
    });

    // Run the worker
    info!("Starting task command processor...");
    let worker = StreamWorker::<TaskCommand, _>::new(redis, processor, worker_config);
    worker
        .run(shutdown_rx)
        .await
        .map_err(|e| eyre::eyre!("{}", e))?;

    info!("Tasks worker service stopped");
    Ok(())
}

/// Wait for a shutdown signal (SIGINT or SIGTERM)
async fn shutdown_signal() -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating shutdown...");
        },
    }

    Ok(())
}
