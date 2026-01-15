//! Vector gRPC Service
//!
//! A microservice for vector storage, search, and embedding operations via gRPC.
//!
//! ## Architecture
//!
//! ```text
//! Client (TypeScript/Rust)
//!   ↓ (gRPC with Zstd compression)
//! VectorServiceImpl (service.rs)
//!   ↓ (proto ↔ domain conversions)
//! VectorService (domain layer)
//!   ↓ (business logic)
//! ┌─────────────┬─────────────────┐
//! │ QdrantRepo  │ EmbeddingProvider│
//! └─────────────┴─────────────────┘
//!   ↓                  ↓
//! Qdrant           OpenAI API
//! ```
//!
//! ## Features
//!
//! - **Multi-tenancy**: Project-based isolation with optional namespaces
//! - **Vector Operations**: Upsert, search, get, delete (batch support)
//! - **Embedding Generation**: OpenAI text-embedding-3-* models
//! - **Combined Operations**: Text → embed → store/search in one call
//! - **Recommendations**: Similar item discovery
//!
//! ## Modules
//!
//! - `server`: Server initialization and lifecycle
//! - `service`: gRPC service implementation (VectorServiceImpl)

pub mod server;
pub mod service;

// Re-export for convenience
pub use server::run;
pub use service::VectorServiceImpl;
