//! Application state management.
//!
//! This module defines the shared application state passed to all request handlers.
//! The state contains:
//! - Configuration
//! - gRPC client connections
//! - Database connections (PostgreSQL, Redis)

use rpc::tasks::tasks_service_client::TasksServiceClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Channel;

/// Shared application state.
///
/// This struct is cloned for each handler (inexpensive Arc clones), providing access to:
/// - Application configuration
/// - gRPC tasks service client
/// - PostgreSQL database connection pool
/// - Redis connection manager
#[derive(Clone)]
pub struct AppState {
    /// Application configuration loaded from environment variables
    pub config: crate::config::Config,
    /// gRPC client for the task service (wrapped in Arc<RwLock> for safe concurrent access)
    pub tasks_client: Arc<RwLock<TasksServiceClient<Channel>>>,
    /// PostgreSQL database connection pool
    pub db: database::postgres::DatabaseConnection,
    /// Redis connection manager
    pub redis: database::redis::ConnectionManager,
}
