//! Redis database connector and utilities
//!
//! Provides connection management and Redis-specific helpers.

mod config;
mod connector;
mod health;

pub use config::RedisConfig;
pub use connector::{
    connect, connect_from_config, connect_from_config_with_retry, connect_with_retry,
    RedisConnector,
};
pub use health::{check_health, check_health_detailed, check_health_with_command, HealthStatus};

// Re-export redis types for convenience
pub use redis::aio::ConnectionManager;
pub use redis::{AsyncCommands, Client, RedisResult};
