//! Server infrastructure module.
//!
//! This module provides:
//! - Application setup with OpenAPI documentation
//! - Health and readiness endpoints
//! - Graceful shutdown coordination
//! - Database connection cleanup
//!
//! # Example
//!
//! ```ignore
//! use axum_helpers::server::{create_app, create_router, health_router};
//! use core_config::{server::ServerConfig, app_info};
//!
//! // Create router with API documentation
//! let router = create_router::<ApiDoc>(api_routes).await?;
//!
//! // Add health endpoints
//! let app = router.merge(health_router(app_info!()));
//!
//! // Start server with graceful shutdown
//! create_app(app, &ServerConfig::default()).await?;
//! ```

pub mod app;
pub mod cleanup;
pub mod health;
pub mod shutdown;

// Re-export commonly used types and functions
pub use app::{create_app, create_production_app, create_router};
pub use cleanup::{close_postgres, close_redis, CleanupCoordinator};
pub use health::{health_router, run_health_checks, HealthCheckFuture, HealthResponse, ReadyResponse};
pub use shutdown::{shutdown_signal, ShutdownCoordinator};
