//! Cloud pricing providers
//!
//! This module contains implementations for fetching pricing data from
//! AWS, Azure, and GCP.

pub mod aws;
pub mod azure;
pub mod gcp;

use async_trait::async_trait;
use domain_pricing::{CloudProvider, CreatePriceEntry, ResourceType};
use thiserror::Error;

pub use aws::AwsPricingProvider;
pub use azure::AzurePricingProvider;
pub use gcp::GcpPricingProvider;

/// Error type for pricing provider operations
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("API error: {0}")]
    ApiError(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// Trait for cloud pricing providers
#[async_trait]
pub trait PricingProvider: Send + Sync {
    /// Get the cloud provider type
    fn provider(&self) -> CloudProvider;

    /// Get the provider name
    fn name(&self) -> &'static str;

    /// Check if the provider is configured and ready
    fn is_configured(&self) -> bool;

    /// Collect pricing data for specified resource types and regions
    async fn collect_prices(
        &self,
        resource_types: &[ResourceType],
        regions: &[String],
    ) -> ProviderResult<Vec<CreatePriceEntry>>;

    /// Health check - verify API connectivity
    async fn health_check(&self) -> ProviderResult<bool>;

    /// Get supported regions for this provider
    fn supported_regions(&self) -> Vec<String>;
}

/// Registry of all pricing providers
pub struct ProviderRegistry {
    providers: Vec<Box<dyn PricingProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn PricingProvider>) {
        self.providers.push(provider);
    }

    pub fn get_provider(&self, cloud_provider: CloudProvider) -> Option<&dyn PricingProvider> {
        self.providers
            .iter()
            .find(|p| p.provider() == cloud_provider)
            .map(|p| p.as_ref())
    }

    pub fn get_configured_providers(&self) -> Vec<&dyn PricingProvider> {
        self.providers
            .iter()
            .filter(|p| p.is_configured())
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn all_providers(&self) -> &[Box<dyn PricingProvider>] {
        &self.providers
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
