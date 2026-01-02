//! Medium MCP Server
//!
//! An MCP (Model Context Protocol) server for fetching and parsing Medium articles.
//! Demonstrates advanced Rust patterns:
//! - Typestate pattern for client lifecycle
//! - Newtype pattern for domain types
//! - Sealed traits for controlled extension
//! - Error context pattern for rich error information
//! - Cow for efficient string handling

mod client;
mod error;
mod mcp;
mod parser;
mod types;

pub use client::{MediumClient, Ready, Uninitialized};
pub use error::{Error, Result};
pub use mcp::{McpHandler, McpRequest, McpResponse, Tool};
pub use parser::ArticleParser;
pub use types::{Article, ArticleId, Author, AuthorId, Tag};
