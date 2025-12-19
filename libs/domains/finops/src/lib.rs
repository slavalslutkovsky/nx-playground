//! FinOps Domain
//!
//! This module provides a complete domain implementation for the FinOps AI assistant.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   Orchestrator  │  ← AI agent routing and coordination
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │     Service     │  ← Business logic, session management
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │   Repository    │  ← Data access (trait + implementations)
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │     Models      │  ← Entities, DTOs, enums
//! └─────────────────┘
//! ```
//!
//! # Features
//!
//! - Chat session management with conversation history
//! - Price comparison and optimization tools
//! - Cloud resource exploration
//! - AI-powered recommendations
//! - SSE streaming responses

pub mod agent;
pub mod entity;
pub mod error;
pub mod handlers;
pub mod models;
pub mod postgres;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use error::{FinopsError, FinopsResult};
pub use models::{
    ChatContext, ChatMessage, ChatRequest, ChatResponse, ChatSession, CloudAccount,
    CloudAccountStatus, CloudResource, MessageRole, Recommendation, RecommendationStatus,
    RecommendationType, SessionStatus,
};
pub use postgres::PgFinopsRepository;
pub use repository::FinopsRepository;
pub use service::FinopsService;

// Re-export agent types
pub use agent::{FinopsOrchestrator, FinopsTool};

// Re-export handler types
pub use handlers::FinopsState;

// Re-export ApiResource trait for accessing generated constants
pub use core_proc_macros::ApiResource;
