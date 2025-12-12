//! # Axum Helpers
//!
//! A collection of utilities, middleware, and helpers for building Axum web applications.
//!
//! ## Modules
//!
//! - **[`auth`]**: JWT authentication with Redis-backed whitelist/blacklist
//! - **[`server`]**: Server setup, health checks, graceful shutdown
//! - **[`http`]**: HTTP middleware (CORS, CSRF, security headers)
//! - **[`errors`]**: Structured error responses with error codes
//! - **[`extractors`]**: Custom extractors (UUID path, validated JSON)
//! - **[`audit`]**: Audit logging for security and compliance
//!
//! ## Quick Start
//!
//! ```ignore
//! use axum::Router;
//! use axum_helpers::server::{create_app, create_router};
//! use core_config::server::ServerConfig;
//! use utoipa::OpenApi;
//!
//! #[derive(OpenApi)]
//! #[openapi(paths())]
//! struct ApiDoc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let api_routes = Router::new(); // Add your routes
//!     let router = create_router::<ApiDoc>(api_routes).await?;
//!
//!     let config = ServerConfig::default();
//!     create_app(router, &config).await?;
//!     Ok(())
//! }
//! ```

// Domain modules
pub mod auth;
pub mod server;
pub mod http;
pub mod errors;
pub mod extractors;
pub mod audit;

// Re-export auth types
pub use auth::{
    JwtClaims, JwtConfig, JwtRedisAuth, RedisAuthStore,
    jwt_auth_middleware, optional_jwt_auth_middleware,
    ACCESS_TOKEN_TTL, REFRESH_TOKEN_TTL,
};

// Re-export server types
pub use server::{
    create_app, create_production_app, create_router,
    health_router, run_health_checks, HealthCheckFuture, HealthResponse, ReadyResponse,
    shutdown_signal, ShutdownCoordinator, CleanupCoordinator,
};

// Re-export HTTP middleware
pub use http::{
    create_cors_layer, create_permissive_cors_layer,
    csrf_validation_middleware, security_headers,
};

// Re-export error types
pub use errors::{AppError, ErrorCode, ErrorResponse};

// Re-export extractors
pub use extractors::{UuidPath, ValidatedJson};

// Re-export audit types
pub use audit::{
    extract_ip_from_headers, extract_ip_from_socket, extract_user_agent,
    AuditEvent, AuditOutcome,
};
