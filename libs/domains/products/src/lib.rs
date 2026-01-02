//! Products Domain
//!
//! This module provides a complete domain implementation for managing products using MongoDB.
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
//! use domain_products::{
//!     handlers,
//!     mongodb::MongoProductRepository,
//!     service::ProductService,
//! };
//! use mongodb::Client;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a MongoDB client
//! let client = Client::with_uri_str("mongodb://localhost:27017").await?;
//! let db = client.database("mydb");
//!
//! // Create a repository and service
//! let repository = MongoProductRepository::new(&db);
//! let service = ProductService::new(repository);
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
pub use error::{ProductError, ProductResult};
pub use handlers::ApiDoc;
pub use models::{
    CreateProduct, Product, ProductCategory, ProductFilter, ProductImage, ProductStatus,
    ReservationResult, StockAdjustment, StockReservation, UpdateProduct,
};
pub use mongodb::MongoProductRepository;
pub use repository::ProductRepository;
pub use service::ProductService;
