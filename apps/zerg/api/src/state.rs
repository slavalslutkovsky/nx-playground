//! Application state management.
//!
//! This module defines the shared application state passed to all request handlers.
//! The state contains:
//! - Configuration
//! - gRPC client connections
//! - Database connections (PostgreSQL, Redis)

use axum_helpers::JwtRedisAuth;
use rpc::tasks::tasks_service_client::TasksServiceClient;
use rpc::vector::vector_service_client::VectorServiceClient;
// use rpc::agent::agent_service_client::AgentServiceClient;  // Disabled for now
use tonic::transport::Channel;
/// Shared application state.
///
/// This struct is cloned for each handler (inexpensive Arc clones), providing access to:
/// - Application configuration
/// - gRPC tasks service client (cheap to clone, shares underlying connection)
/// - PostgreSQL database connection pool (SeaORM)
/// - Redis connection manager
/// - JWT authentication (hybrid JWT + Redis)
#[derive(Clone)]
pub struct AppState {
    /// Application configuration loaded from environment variables
    pub config: crate::config::Config,
    /// gRPC client for the task service (cloneable, shares HTTP/2 connection pool)
    /// No lock needed - cloning is cheap and thread-safe
    pub tasks_client: TasksServiceClient<Channel>,
    /// gRPC client for the vector service (cloneable, shares HTTP/2 connection pool)
    pub vector_client: VectorServiceClient<Channel>,
    /// gRPC client for the agent service (cloneable, shares HTTP/2 connection pool)
    /// Provides access to AI agent invocation via rag-agent and other agents
    // pub agent_client: AgentServiceClient<Channel>,

    /// PostgreSQL database connection pool (SeaORM)
    pub db: database::postgres::DatabaseConnection,
    /// Redis connection manager
    pub redis: database::redis::ConnectionManager,
    /// JWT + Redis hybrid authentication
    pub jwt_auth: JwtRedisAuth,
}
