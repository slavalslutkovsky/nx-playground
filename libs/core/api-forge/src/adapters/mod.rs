//! API Source Adapters
//!
//! Adapters convert various API definition formats into the unified schema.

pub mod openapi;
pub mod openapi_simple;

pub use openapi::OpenApiAdapter;
pub use openapi_simple::SimpleOpenApiAdapter;

use crate::ApiRegistry;
use crate::error::Result;
use async_trait::async_trait;

/// Trait for adapters that can populate an API registry
#[async_trait]
pub trait ApiAdapter: Send + Sync {
    /// Parse the source and populate a registry
    async fn parse(&self) -> Result<ApiRegistry>;

    /// Get a description of this adapter
    fn description(&self) -> &str;
}
