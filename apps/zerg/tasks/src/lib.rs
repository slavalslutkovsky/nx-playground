//! Unified gRPC Service
//!
//! A microservice supporting multiple gRPC services (tasks, vector).
//!
//! ## Architecture
//!
//! ```text
//! Client
//!   ↓ (gRPC with Zstd compression)
//! Service Implementations (service.rs, vector_service.rs)
//!   ↓ (proto ↔ domain conversions via From/TryFrom traits)
//! Domain Services (domain layer)
//!   ↓ (business logic)
//! Repositories (persistence)
//!   ↓
//! PostgreSQL / Qdrant
//! ```
//!
//! ## Modules
//!
//! - `server`: Server initialization and lifecycle
//! - `service`: Tasks gRPC service implementation
//! - `vector_service`: Vector gRPC service implementation

pub mod server;
pub mod service;
pub mod vector_service;

pub use server::run;
pub use service::TasksServiceImpl;
pub use vector_service::VectorServiceImpl;
