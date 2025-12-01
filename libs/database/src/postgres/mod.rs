//! PostgreSQL database connector and utilities
//!
//! Provides connection management, migration running, and PostgreSQL-specific helpers.

mod config;
mod connector;
mod health;

pub use config::PostgresConfig;
pub use connector::{
    connect, connect_from_config, connect_from_config_with_retry, connect_with_options,
    connect_with_retry, run_migrations,
};
pub use health::{check_health, check_health_detailed, check_health_with_query, HealthStatus};

// Re-export SeaORM types for convenience
pub use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
pub use sea_orm_migration::MigratorTrait;
