//! API Forge - Unified API Documentation & Analysis Library
//!
//! This library provides tools for:
//! - Aggregating APIs from multiple sources (OpenAPI, gRPC, GraphQL)
//! - Generating unified documentation
//! - AI-powered security validation and best practices analysis
//! - Serving `/help` endpoints with API metadata

pub mod adapters;
pub mod error;
pub mod handlers;
pub mod registry;
pub mod schema;

pub use error::{ApiForgeError, Result};
pub use registry::ApiRegistry;
pub use schema::{ApiEndpoint, ApiSchema, HttpMethod};

struct Aris<'a> {
  name: &'a str,
}
