//! Items Domain
//!
//! This module provides a complete domain implementation for managing items using MongoDB.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │  Handlers   │  ← HTTP endpoints
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │   Service   │  ← Business logic, validation
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │ Repository  │  ← Data access (trait + MongoDB implementation)
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │   Models    │  ← Entities, DTOs
//! └─────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use domain_items::{
//!     handlers,
//!     mongodb::MongoItemRepository,
//!     service::ItemService,
//! };
//! use mongodb::Client;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a MongoDB client
//! let client = Client::with_uri_str("mongodb://localhost:27017").await?;
//! let db = client.database("mydb");
//!
//! // Create a repository and service
//! let repository = MongoItemRepository::new(db);
//! let service = ItemService::new(repository);
//!
//! // Create Axum router
//! let router = handlers::router(service);
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod handlers;
pub mod models;
pub mod mongodb;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use error::{ItemError, ItemResult};
pub use handlers::ApiDoc;
pub use models::{CreateItem, Item, ItemFilter, ItemStatus, UpdateItem};
pub use mongodb::MongoItemRepository;
pub use repository::ItemRepository;
pub use service::ItemService;
