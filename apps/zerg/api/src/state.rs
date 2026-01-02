//! Application state management.
//!
//! This module defines the shared application state passed to all request handlers.
//! The state contains:
//! - Configuration
//! - gRPC client connections
//! - Database connections (PostgreSQL, Redis)
//! - Optional vector/graph database clients (Qdrant, Neo4j, ArangoDB, Milvus)
//! - Optional NATS event publisher

use axum_helpers::JwtRedisAuth;
use rpc::tasks::tasks_service_client::TasksServiceClient;
use tonic::transport::Channel;

// Re-export database client states for convenience
pub use crate::api::arangodb::ArangoState;
pub use crate::api::milvus::MilvusState;
pub use crate::api::neo4j::Neo4jState;
pub use crate::api::qdrant::QdrantState;
pub use crate::events::EventPublisher;

/// Shared application state.
///
/// This struct is cloned for each handler (inexpensive Arc clones), providing access to:
/// - Application configuration
/// - gRPC tasks service client (cheap to clone, shares underlying connection)
/// - PostgreSQL database connection pool (SeaORM)
/// - Redis connection manager
/// - JWT authentication (hybrid JWT + Redis)
/// - Optional vector/graph database clients
#[derive(Clone)]
pub struct AppState {
    /// Application configuration loaded from environment variables
    pub config: crate::config::Config,
    /// gRPC client for the task service (cloneable, shares HTTP/2 connection pool)
    /// No lock needed - cloning is cheap and thread-safe
    pub tasks_client: TasksServiceClient<Channel>,
    /// PostgreSQL database connection pool (SeaORM)
    pub db: database::postgres::DatabaseConnection,
    /// Redis connection manager
    pub redis: database::redis::ConnectionManager,
    /// JWT + Redis hybrid authentication
    pub jwt_auth: JwtRedisAuth,

    // Vector & Graph database clients (optional)
    /// Qdrant vector database client (set QDRANT_URL to enable)
    pub qdrant: Option<QdrantState>,
    /// Neo4j graph database client (set NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD to enable)
    pub neo4j: Option<Neo4jState>,
    /// ArangoDB multi-model database client (set ARANGO_URL, ARANGO_USER, ARANGO_PASSWORD, ARANGO_DATABASE to enable)
    pub arangodb: Option<ArangoState>,
    /// Milvus vector database client (set MILVUS_URL to enable)
    pub milvus: Option<MilvusState>,

    // Messaging
    /// NATS event publisher (set NATS_URL to enable)
    pub events: Option<EventPublisher>,
}
