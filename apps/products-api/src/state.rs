//! Application state management

use mongodb::{Client, Database};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::Config,
    pub mongo_client: Client,
    pub db: Database,
}
