//! Tasks gRPC Service
//!
//! A microservice for managing tasks via gRPC.
//!
//! ## Architecture
//!
//! ```text
//! Client
//!   ↓ (gRPC with Zstd compression)
//! TasksServiceImpl (service.rs)
//!   ↓ (proto ↔ domain conversions via From/TryFrom traits)
//! TaskService (domain layer)
//!   ↓ (business logic)
//! PgTaskRepository (persistence)
//!   ↓
//! PostgreSQL
//! ```
//!
//! ## Modules
//!
//! - `server`: Server initialization and lifecycle
//! - `service`: gRPC service implementation (TasksServiceImpl)

pub mod server;
pub mod service;

// Re-export for convenience
pub use server::run;
pub use service::TasksServiceImpl;
