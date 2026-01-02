//! Error types for API Forge

use thiserror::Error;

/// Result type for API Forge operations
pub type Result<T> = std::result::Result<T, ApiForgeError>;

/// Errors that can occur in API Forge
#[derive(Debug, Error)]
pub enum ApiForgeError {
    /// Failed to parse OpenAPI specification
    #[error("Failed to parse OpenAPI spec: {0}")]
    OpenApiParse(String),

    /// Failed to fetch remote specification
    #[error("Failed to fetch spec from URL: {0}")]
    FetchError(String),

    /// Schema not found
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    /// Endpoint not found
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),

    /// Invalid path pattern
    #[error("Invalid path pattern: {0}")]
    InvalidPath(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}
