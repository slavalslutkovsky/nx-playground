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
//!     repository::InMemoryProjectRepository,
//!     service::ProjectService,
//! };
//!
//! // Create repository and service
//! let repository = InMemoryProjectRepository::new();
//! let service = ProjectService::new(repository);
//!
//! // Create Axum router
//! let router = handlers::router(service);
//! ```

pub mod error;
pub mod handlers;
pub mod models;
pub mod postgres;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use error::{ProjectError, ProjectResult};
pub use models::{
    CloudProvider, CreateProject, Environment, Project, ProjectFilter, ProjectStatus, Tag,
    UpdateProject,
};
pub use postgres::PgProjectRepository;
pub use repository::{InMemoryProjectRepository, ProjectRepository};
pub use service::ProjectService;
