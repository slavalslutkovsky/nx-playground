//! Authentication and authorization module.
//!
//! This module provides:
//! - JWT token creation and verification with Redis-backed whitelist/blacklist
//! - Session user types for axum-login integration
//! - Authentication middleware for protected routes
//!
//! # Example
//!
//! ```ignore
//! use axum_helpers::auth::{JwtConfig, JwtRedisAuth, jwt_auth_middleware};
//! use core_config::FromEnv;
//!
//! // Load config and create auth instance
//! let config = JwtConfig::from_env()?;
//! let auth = JwtRedisAuth::new(redis_manager, &config)?;
//!
//! // Protect routes with JWT middleware
//! let protected = Router::new()
//!     .route("/api/protected", get(handler))
//!     .layer(axum::middleware::from_fn_with_state(auth, jwt_auth_middleware));
//! ```

pub mod config;
pub mod jwt;
pub mod middleware;
pub mod store;

// Re-export commonly used types
pub use config::JwtConfig;
pub use jwt::{ACCESS_TOKEN_TTL, JwtClaims, JwtRedisAuth, REFRESH_TOKEN_TTL};
pub use middleware::{jwt_auth_middleware, optional_jwt_auth_middleware};
pub use store::RedisAuthStore;
