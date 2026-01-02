//! MongoDB database connector and utilities
//!
//! Provides connection management and MongoDB-specific helpers.

mod config;
mod connector;
mod health;

pub use config::MongoConfig;
pub use connector::{
    connect, connect_from_config, connect_from_config_with_retry, connect_with_retry,
};
pub use health::{HealthStatus, check_health, check_health_detailed};

// Re-export MongoDB types for convenience
pub use mongodb::{Client, Collection, Database};
