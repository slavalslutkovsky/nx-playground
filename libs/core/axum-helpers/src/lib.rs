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

pub mod audit;
pub mod cleanup;
pub mod config;
pub mod errors;
pub mod extractors;
pub mod health;
pub mod jwt_redis_auth;
pub mod middleware;
pub mod redis_auth_store;
pub mod server;
pub mod shutdown;

// Re-export commonly used types
pub use cleanup::CleanupCoordinator;
pub use config::JwtConfig;
pub use errors::{AppError, ErrorCode, ErrorResponse};
pub use health::{HealthResponse, ReadyResponse};
pub use jwt_redis_auth::{JwtClaims, JwtRedisAuth, ACCESS_TOKEN_TTL, REFRESH_TOKEN_TTL};
pub use redis_auth_store::RedisAuthStore;
pub use server::{create_app, create_production_app, create_router};
pub use shutdown::{shutdown_signal, ShutdownCoordinator};

// Re-export middleware functions
pub use middleware::cors::{create_cors_layer, create_permissive_cors_layer};
pub use middleware::csrf::csrf_validation_middleware;
pub use middleware::security::security_headers;

// Re-export extractors
pub use extractors::{UuidPath, ValidatedJson};

// Re-export audit types
pub use audit::{
    extract_ip_from_headers, extract_ip_from_socket, extract_user_agent, AuditEvent, AuditOutcome,
};
