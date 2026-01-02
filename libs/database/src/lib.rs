//! Database library providing connectors and utilities for PostgreSQL, Redis, MongoDB, and Cassandra
//!
//! This library provides a unified interface for connecting to and managing database
//! connections across different database types.
//!
//! # Features
//!
//! - `postgres` (default) - PostgreSQL support with SeaORM
//! - `redis` (default) - Redis support
//! - `mongodb` - MongoDB support
//! - `cassandra` - Cassandra/ScyllaDB support
//! - `config` - Configuration support with `core_config::FromEnv`
//! - `all` - All database features
//!
//! # Examples
//!
//! ## PostgreSQL
//!
//! ```ignore
//! use database::postgres;
//! use my_app::migrator::Migrator;
//!
//! let db = postgres::connect("postgresql://user:pass@localhost/db").await?;
//! postgres::run_migrations::<Migrator>(&db, "my_app").await?;
//! ```
//!
//! ## Redis
//!
//! ```ignore
//! use database::redis;
//! use redis::AsyncCommands;
//!
//! let mut conn = redis::connect("redis://127.0.0.1:6379").await?;
//! conn.set::<_, _, ()>("key", "value").await?;
//! ```
//!
//! ## MongoDB
//!
//! ```ignore
//! use database::mongodb;
//!
//! let client = mongodb::connect("mongodb://localhost:27017").await?;
//! let db = client.database("mydb");
//! let collection = db.collection::<Document>("items");
//! ```
//!
//! ## Cassandra/ScyllaDB
//!
//! ```ignore
//! use database::cassandra;
//!
//! let session = cassandra::connect(&["127.0.0.1:9042"]).await?;
//! session.query_unpaged("SELECT * FROM users", &[]).await?;
//!
//! // With configuration
//! let config = cassandra::CassandraConfig::with_keyspace(
//!     vec!["127.0.0.1:9042"],
//!     "mykeyspace"
//! );
//! let session = cassandra::connect_from_config(&config).await?;
//! ```

// Always available modules
pub mod common;

// Repository abstraction (requires postgres feature since it uses SeaORM)
#[cfg(feature = "postgres")]
pub mod repository;

// Database-specific modules (conditional based on features)
#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "mongodb")]
pub mod mongodb;

#[cfg(feature = "cassandra")]
pub mod cassandra;

// Re-exports for convenience
pub use common::{DatabaseError, DatabaseResult};

#[cfg(feature = "postgres")]
pub use repository::{BaseRepository, UuidEntity};

// Type-state pattern example (kept for documentation purposes)
#[allow(dead_code)]
fn run_all(tasks: Vec<Box<dyn Fn() + Send>>) {
    for t in tasks {
        t();
    }
}

use std::marker::PhantomData;

#[allow(dead_code)]
struct Disconnected;

#[allow(dead_code)]
struct Connected;

#[allow(dead_code)]
struct Client<State> {
    addr: String,
    _marker: PhantomData<State>,
}

#[allow(dead_code)]
impl Client<Disconnected> {
    fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            _marker: PhantomData,
        }
    }
    fn connect(self) -> Client<Connected> {
        Client {
            addr: self.addr,
            _marker: PhantomData,
        }
    }
}

#[allow(dead_code)]
impl Client<Connected> {
    fn send(&self, msg: &str) {
        println!("send to {}: {}", self.addr, msg);
    }
}
