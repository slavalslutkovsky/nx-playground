//! Projects Domain
//!
//! This module provides a complete domain implementation for managing cloud projects.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │  Handlers   │  ← HTTP/gRPC endpoints
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │   Service   │  ← Business logic, validation
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │ Repository  │  ← Data access (trait + implementations)
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │   Models    │  ← Entities, DTOs, enums
//! └─────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use domain_projects::{
//!     handlers,
//!     postgres::PgProjectRepository,
//!     service::ProjectService,
//! };
//! use sea_orm::Database;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a database connection
//! let db = Database::connect("postgres://...").await?;
//!
//! // Create a repository and service
//! let repository = PgProjectRepository::new(db);
//! let service = ProjectService::new(repository);
//!
//! // Create Axum router
//! let router = handlers::router(service);
//! # Ok(())
//! # }
//! ```

pub mod entity;
pub mod error;
pub mod handlers;
pub mod models;
pub mod postgres;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use error::{ProjectError, ProjectResult};
pub use handlers::ApiDoc;
pub use models::{
    CloudProvider, CreateProject, Environment, Project, ProjectFilter, ProjectStatus, Tag,
    UpdateProject,
};
pub use postgres::PgProjectRepository;
pub use repository::ProjectRepository;
pub use service::ProjectService;

// Re-export ApiResource trait for accessing generated constants
pub use core_proc_macros::ApiResource;
