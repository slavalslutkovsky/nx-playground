//! Cassandra/ScyllaDB database connector and utilities
//!
//! Provides connection management and Cassandra-specific helpers.
//! Uses the `scylla` driver which is compatible with both Apache Cassandra
//! and ScyllaDB.
//!
//! # Example
//!
//! ```ignore
//! use database::cassandra::{connect, CassandraConfig};
//!
//! // Simple connection
//! let session = connect(&["127.0.0.1:9042"]).await?;
//!
//! // With config
//! let config = CassandraConfig::with_keyspace(vec!["127.0.0.1:9042"], "mykeyspace")
//!     .with_datacenter("dc1")
//!     .with_credentials("user", "password");
//! let session = connect_from_config(&config).await?;
//!
//! // Execute queries
//! session.query_unpaged("SELECT * FROM users", &[]).await?;
//! ```

mod config;
mod connector;
mod health;

pub use config::CassandraConfig;
pub use connector::{
    CassandraError, CassandraSession, connect, connect_from_config, connect_from_config_with_retry,
    connect_with_retry, create_keyspace_if_not_exists, use_keyspace,
};
pub use health::{
    ClusterInfo, HealthStatus, check_health, check_health_detailed, get_cluster_info,
};

// Re-export scylla types for convenience
pub use scylla::client::session::Session;
pub use scylla::client::session_builder::SessionBuilder;
pub use scylla::serialize::value::SerializeValue;
pub use scylla::value::CqlValue;
