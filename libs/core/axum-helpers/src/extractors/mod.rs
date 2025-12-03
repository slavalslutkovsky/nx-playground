//! Custom extractors for Axum handlers.
//!
//! This module provides reusable extractors that reduce boilerplate
//! and standardize error handling across your API.

pub mod uuid_path;
pub mod validated_json;

pub use uuid_path::UuidPath;
pub use validated_json::ValidatedJson;
