//! Error handling with context pattern
//!
//! Each layer adds context to errors as they bubble up,
//! creating a rich error trail for debugging.

use std::fmt;

/// Result type alias for Medium MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for Medium MCP operations
#[derive(Debug)]
pub enum Error {
    /// Failed to fetch article from Medium
    Fetch { url: String, source: reqwest::Error },
    /// Failed to parse HTML content
    Parse { context: String, details: String },
    /// Article not found
    NotFound { article_id: String },
    /// Rate limited by Medium
    RateLimited { retry_after: Option<u64> },
    /// Invalid MCP request
    InvalidRequest { message: String },
    /// Unknown tool requested
    UnknownTool { name: String },
    /// Serialization/deserialization error
    Serialization {
        context: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fetch { url, source } => {
                write!(f, "Failed to fetch article from {}: {}", url, source)
            }
            Error::Parse { context, details } => {
                write!(f, "Parse error in {}: {}", context, details)
            }
            Error::NotFound { article_id } => {
                write!(f, "Article not found: {}", article_id)
            }
            Error::RateLimited { retry_after } => match retry_after {
                Some(secs) => write!(f, "Rate limited, retry after {} seconds", secs),
                None => write!(f, "Rate limited by Medium"),
            },
            Error::InvalidRequest { message } => {
                write!(f, "Invalid MCP request: {}", message)
            }
            Error::UnknownTool { name } => {
                write!(f, "Unknown tool: {}", name)
            }
            Error::Serialization { context, source } => {
                write!(f, "Serialization error in {}: {}", context, source)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fetch { source, .. } => Some(source),
            Error::Serialization { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> ErrorContext<T> for std::result::Result<T, reqwest::Error> {
    fn with_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::Fetch {
            url: context.into(),
            source: e,
        })
    }
}

impl<T> ErrorContext<T> for std::result::Result<T, serde_json::Error> {
    fn with_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::Serialization {
            context: context.into(),
            source: e,
        })
    }
}
