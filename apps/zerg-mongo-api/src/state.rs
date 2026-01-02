//! Application state management.
//!
//! This module defines the shared application state passed to all request handlers.
//! The state contains:
//! - Configuration
//! - MongoDB client

use mongodb::{Client, Database};

/// Shared application state.
///
/// This struct is cloned for each handler (inexpensive Arc clones), providing access to:
/// - Application configuration
/// - MongoDB client and database
#[derive(Clone)]
pub struct AppState {
    /// Application configuration loaded from environment variables
    pub config: crate::config::Config,
    /// MongoDB client (cloneable, shares underlying connection pool)
    pub mongo_client: Client,
    /// MongoDB database instance
    pub db: Database,
}
