//! HTTP Handlers for API Forge
//!
//! Provides Axum handlers for serving API documentation and help endpoints.

pub mod help;

pub use help::{HelpState, help_router};
