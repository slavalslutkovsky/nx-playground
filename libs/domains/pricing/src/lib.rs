//! Pricing Domain
//!
//! This module provides a complete domain implementation for managing cloud pricing data.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Service   │  ← Business logic, price comparison
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

pub mod ai_recommender;
pub mod cncf_client;
pub mod cncf_models;
pub mod conversions;
pub mod entity;
pub mod error;
pub mod handlers;
pub mod models;
pub mod postgres;
pub mod repository;
pub mod service;
pub mod tco_calculator;

// Re-export commonly used types
pub use error::{PricingError, PricingResult};
pub use models::{
    CloudProvider, CreatePriceEntry, Currency, Money, PriceEntry, PriceFilter, PricingUnit,
    ResourceType, UpdatePriceEntry,
};
pub use postgres::PgPricingRepository;
pub use repository::PricingRepository;
pub use service::PricingService;

// Re-export CNCF types
pub use cncf_models::{
    CncfMaturity, CncfTool, CncfToolCategory, CostRecommendation, DeploymentMode,
    InfrastructureCostComparison, ManagedServiceEquivalent, OpsHoursEstimate,
    ResourceRequirements, TcoCalculationRequest, TcoCalculationResult,
};

// Re-export CNCF client types
pub use cncf_client::{
    CategoryRecommendationResponse, CncfCategoryGroup, CncfLandscapeClient, CncfToolEnriched,
    CncfToolsResponse, GitHubStats, ToolRecommendation,
};

// Re-export AI recommender
pub use ai_recommender::{AiRecommender, HeuristicRecommender};

// Re-export ApiResource trait for accessing generated constants
pub use core_proc_macros::ApiResource;
