use async_trait::async_trait;
use uuid::Uuid;

use crate::error::PricingResult;
use crate::models::{CreatePriceEntry, PriceEntry, PriceFilter, UpdatePriceEntry};

/// Repository trait for Price persistence
///
/// This trait defines the data access interface for cloud pricing data.
/// Implementations can use different storage backends (PostgreSQL, etc.)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PricingRepository: Send + Sync {
    /// Create a new price entry
    async fn create(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry>;

    /// Create multiple price entries in bulk
    async fn create_many(&self, inputs: Vec<CreatePriceEntry>) -> PricingResult<Vec<PriceEntry>>;

    /// Get a price entry by ID
    async fn get_by_id(&self, id: Uuid) -> PricingResult<Option<PriceEntry>>;

    /// Get a price entry by SKU, provider, and region (unique combination)
    async fn get_by_sku(
        &self,
        sku: &str,
        provider: crate::models::CloudProvider,
        region: &str,
    ) -> PricingResult<Option<PriceEntry>>;

    /// List price entries with optional filters
    async fn list(&self, filter: PriceFilter) -> PricingResult<Vec<PriceEntry>>;

    /// Update an existing price entry
    async fn update(&self, id: Uuid, input: UpdatePriceEntry) -> PricingResult<PriceEntry>;

    /// Delete a price entry by ID
    async fn delete(&self, id: Uuid) -> PricingResult<bool>;

    /// Upsert a price entry (insert or update based on SKU+provider+region)
    async fn upsert(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry>;

    /// Count all price entries
    async fn count(&self) -> PricingResult<usize>;

    /// Count price entries by provider
    async fn count_by_provider(
        &self,
        provider: crate::models::CloudProvider,
    ) -> PricingResult<usize>;

    /// Get distinct regions for a provider
    async fn get_regions_for_provider(
        &self,
        provider: crate::models::CloudProvider,
    ) -> PricingResult<Vec<String>>;

    /// Delete expired prices (where expiration_date < now)
    async fn delete_expired(&self) -> PricingResult<usize>;
}
