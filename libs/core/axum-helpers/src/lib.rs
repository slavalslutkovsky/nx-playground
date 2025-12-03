//! # Axum Helpers
//!
//! A collection of utilities, middleware, and helpers for building Axum web applications.
//!
//! ## Features
//!
//! - **Server Management**: Easy server setup with graceful shutdown
//! - **Middleware**: Security headers, CORS, CSRF protection
//! - **Error Handling**: Structured error responses with proper HTTP status codes
//! - **Health Checks**: Built-in health and readiness endpoints
//! - **OpenAPI Documentation**: Integrated Swagger UI, ReDoc, RapiDoc, and Scalar
//! - **Extractors**: Custom extractors for common use cases
//!
//! ## Quick Start
//!
//! ```ignore
//! use axum::Router;
//! use axum_helpers::server::{create_app, create_router};
//! use core_config::server::ServerConfig;
//! use utoipa::OpenApi;
//!
//! #[derive(Clone)]
//! struct AppState {}
//!
//! #[derive(OpenApi)]
//! #[openapi(paths())]
//! struct ApiDoc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let state = AppState {};
//!     let api_routes = Router::new(); // Add your routes
//!     let router = create_router::<ApiDoc, _>(state, api_routes).await?;
//!
//!     let config = ServerConfig::default();
//!     create_app(router, &config).await?;
//!     Ok(())
//! }
//! ```

pub mod server;
pub mod shutdown;
pub mod middleware;
pub mod errors;
pub mod health;
pub mod extractors;
pub mod cleanup;
pub mod audit;

// Re-export commonly used types
pub use errors::{AppError, ErrorResponse};
pub use health::{HealthResponse, ReadyResponse};
pub use server::{create_app, create_router, create_production_app};
pub use shutdown::{shutdown_signal, ShutdownCoordinator};
pub use cleanup::CleanupCoordinator;

// Re-export middleware functions
pub use middleware::security::security_headers;
pub use middleware::cors::{create_cors_layer, create_permissive_cors_layer};
pub use middleware::csrf::csrf_validation_middleware;

// Re-export extractors
pub use extractors::{UuidPath, ValidatedJson};

// Re-export audit types
pub use audit::{
    extract_ip_from_headers, extract_ip_from_socket, extract_user_agent, AuditEvent,
    AuditOutcome,
};
