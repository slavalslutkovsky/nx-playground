//! Email Worker Service
//!
//! A background worker that processes email jobs from a Redis stream.
//!
//! ## Architecture
//!
//! ```text
//! Redis Stream (email:jobs)
//!   ↓ (Consumer Group: email_workers)
//! StreamWorker<EmailJob, EmailProcessor>
//!   ↓ (renders templates)
//! TemplateEngine (Handlebars)
//!   ↓ (sends emails)
//! EmailProvider (SendGrid/SMTP)
//!   ↓
//! Email Delivery
//! ```
//!
//! ## Features
//!
//! - Consumer group support for horizontal scaling
//! - Automatic retry with exponential backoff
//! - Dead letter queue for failed jobs
//! - Graceful shutdown handling
//! - Health check endpoint for Kubernetes probes

use axum::Router;
use core_config::{Environment, FromEnv, app_info};
use database::redis::RedisConfig;
use email::{
    EmailJob, EmailProcessor, EmailStream, SendGridProvider, SmtpProvider, TemplateEngine,
};
use eyre::{Result, WrapErr};
use std::sync::Arc;
use stream_worker::{HealthState, StreamWorker, WorkerConfig, full_admin_router, metrics};
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

/// Run the email worker
///
/// This is the main entry point for the worker. It:
/// 1. Sets up structured logging (env-aware: JSON for prod, pretty for dev)
/// 2. Connects to Redis with retry logic
/// 3. Selects the appropriate email provider (SendGrid for prod, SMTP for dev)
/// 4. Starts the worker with graceful shutdown handling
///
/// # Errors
///
/// Returns an error if:
/// - Redis configuration is invalid
/// - Redis connection fails
/// - Email provider configuration is invalid
/// - Worker encounters a fatal error
pub async fn run() -> Result<()> {
    // Initialize tracing (env-aware: JSON for prod, pretty for dev)
    let environment = Environment::from_env();
    core_config::tracing::init_tracing(&environment);

    // Initialize Prometheus metrics
    metrics::init_metrics();

    // App info for health endpoint
    let app_info = app_info!();

    info!(name = %app_info.name, version = %app_info.version, "Starting email worker service");
    info!("Environment: {:?}", environment);

    // Health server port (default 8081)
    // Checks EMAIL_WORKER_HEALTH_PORT first, then HEALTH_PORT, then default
    // Note: Do NOT use PORT as fallback - that's typically used by the main API
    let health_port: u16 = std::env::var("EMAIL_WORKER_HEALTH_PORT")
        .or_else(|_| std::env::var("HEALTH_PORT"))
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .unwrap_or(8081);

    // Load Redis configuration from the environment
    let redis_config = RedisConfig::from_env().wrap_err("Failed to load Redis configuration")?;

    // Connect to Redis with retry logic
    info!("Connecting to Redis...");
    let redis = database::redis::connect_from_config_with_retry(redis_config, None)
        .await
        .wrap_err("Failed to connect to Redis")?;
    info!("Connected to Redis successfully");

    // Create worker configuration from EmailStream definition
    // Note: Disable blocking reads because ConnectionManager uses a single connection,
    // and blocking XREADGROUP prevents other commands (XPENDING, etc.) from executing.
    let worker_config = WorkerConfig::from_stream_def::<EmailStream>().with_blocking(None);
    info!(
        stream = %worker_config.stream_name,
        consumer_group = %worker_config.consumer_group,
        consumer_id = %worker_config.consumer_id,
        poll_interval_ms = %worker_config.poll_interval_ms,
        "Worker configuration loaded"
    );

    // Initialize template engine
    let templates = TemplateEngine::new().wrap_err("Failed to initialize template engine")?;
    info!("Template engine initialized");

    // Set up a shutdown signal
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn shutdown signal handler
    tokio::spawn(async move {
        if let Err(e) = shutdown_signal().await {
            error!("Error waiting for shutdown signal: {}", e);
        }
        let _ = shutdown_tx.send(true);
    });

    // Create the health state
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

    // Select email provider based on environment and run worker
    match environment {
        Environment::Production => {
            info!("Using SendGrid provider for production");
            match SendGridProvider::from_env() {
                Ok(provider) => {
                    let processor = EmailProcessor::new(provider, templates);
                    let worker = StreamWorker::<EmailJob, _>::new(redis, processor, worker_config);
                    worker
                        .run(shutdown_rx)
                        .await
                        .map_err(|e| eyre::eyre!("{}", e))?;
                }
                Err(e) => {
                    error!("Failed to create SendGrid provider: {}", e);
                    return Err(eyre::eyre!(
                        "SendGrid configuration error: {}. Ensure SENDGRID_API_KEY and SENDGRID_FROM_EMAIL are set.",
                        e
                    ));
                }
            }
        }
        Environment::Development => {
            info!("Using SMTP provider for development (Mailpit/MailHog)");
            match SmtpProvider::mailhog() {
                Ok(provider) => {
                    let processor = EmailProcessor::new(provider, templates);
                    let worker = StreamWorker::<EmailJob, _>::new(redis, processor, worker_config);
                    worker
                        .run(shutdown_rx)
                        .await
                        .map_err(|e| eyre::eyre!("{}", e))?;
                }
                Err(e) => {
                    error!("Failed to create SMTP provider: {}", e);
                    return Err(eyre::eyre!(
                        "SMTP configuration error: {}. Ensure SMTP_HOST and SMTP_PORT are accessible.",
                        e
                    ));
                }
            }
        }
    }

    info!("Email worker service stopped");
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
