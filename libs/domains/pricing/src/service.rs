use std::sync::Arc;
use uuid::Uuid;

use crate::error::{PricingError, PricingResult};
use crate::models::{
    CloudProvider, CreatePriceEntry, Money, PriceComparison, PriceEntry, PriceFilter,
    ProviderPrice, ResourceType, UpdatePriceEntry,
};
use crate::repository::PricingRepository;

/// Service for managing cloud pricing data
#[derive(Clone)]
pub struct PricingService<R: PricingRepository> {
    repository: Arc<R>,
}

impl<R: PricingRepository> PricingService<R> {
    /// Create a new pricing service
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Create a new price entry
    pub async fn create(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry> {
        self.repository.create(input).await
    }

    /// Create multiple price entries in bulk
    pub async fn create_many(&self, inputs: Vec<CreatePriceEntry>) -> PricingResult<Vec<PriceEntry>> {
        self.repository.create_many(inputs).await
    }

    /// Get a price entry by ID
    pub async fn get_by_id(&self, id: Uuid) -> PricingResult<PriceEntry> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| PricingError::NotFound(id.to_string()))
    }

    /// List price entries with filters
    pub async fn list(&self, filter: PriceFilter) -> PricingResult<Vec<PriceEntry>> {
        self.repository.list(filter).await
    }

    /// Update a price entry
    pub async fn update(&self, id: Uuid, input: UpdatePriceEntry) -> PricingResult<PriceEntry> {
        self.repository.update(id, input).await
    }

    /// Delete a price entry
    pub async fn delete(&self, id: Uuid) -> PricingResult<bool> {
        self.repository.delete(id).await
    }

    /// Upsert a price entry (insert or update based on SKU+provider+region)
    pub async fn upsert(&self, input: CreatePriceEntry) -> PricingResult<PriceEntry> {
        self.repository.upsert(input).await
    }

    /// Compare prices across providers for similar resources
    pub async fn compare_prices(
        &self,
        resource_type: ResourceType,
        vcpus: Option<i32>,
        memory_gb: Option<i32>,
        regions: Vec<String>,
        providers: Vec<CloudProvider>,
    ) -> PricingResult<Vec<PriceComparison>> {
        // Build filter for each provider
        let providers_to_check = if providers.is_empty() {
            vec![CloudProvider::Aws, CloudProvider::Azure, CloudProvider::Gcp]
        } else {
            providers
        };

        let mut all_prices: Vec<PriceEntry> = Vec::new();

        for provider in &providers_to_check {
            let filter = PriceFilter {
                provider: Some(*provider),
                resource_type: Some(resource_type),
                regions: if regions.is_empty() {
                    None
                } else {
                    Some(regions.join(","))
                },
                limit: 100,
                ..Default::default()
            };

            let prices = self.repository.list(filter).await?;
            all_prices.extend(prices);
        }

        // Filter by vCPUs and memory if specified
        let filtered_prices: Vec<PriceEntry> = all_prices
            .into_iter()
            .filter(|p| {
                let vcpu_match = vcpus
                    .map(|v| {
                        p.attributes
                            .get("vcpu")
                            .and_then(|s| s.parse::<i32>().ok())
                            .is_some_and(|pv| pv == v)
                    })
                    .unwrap_or(true);

                let memory_match = memory_gb
                    .map(|m| {
                        p.attributes
                            .get("memory_gb")
                            .and_then(|s| s.parse::<i32>().ok())
                            .is_some_and(|pm| pm == m)
                    })
                    .unwrap_or(true);

                vcpu_match && memory_match
            })
            .collect();

        // Group by comparison key (e.g., by instance type or specs)
        let comparison_key = format!(
            "{:?}-{}vcpu-{}gb",
            resource_type,
            vcpus.unwrap_or(0),
            memory_gb.unwrap_or(0)
        );

        // Calculate provider prices with monthly estimates
        let provider_prices: Vec<ProviderPrice> = filtered_prices
            .into_iter()
            .map(|price| {
                let monthly_estimate = self.calculate_monthly_estimate(&price);
                ProviderPrice {
                    provider: price.provider,
                    price,
                    monthly_estimate,
                }
            })
            .collect();

        // Find cheapest
        let cheapest = provider_prices
            .iter()
            .min_by_key(|p| p.monthly_estimate.amount)
            .cloned();

        // Calculate potential savings
        let potential_savings = if provider_prices.len() > 1 {
            let max_cost = provider_prices
                .iter()
                .max_by_key(|p| p.monthly_estimate.amount)
                .map(|p| p.monthly_estimate.amount)
                .unwrap_or(0);
            let min_cost = cheapest
                .as_ref()
                .map(|p| p.monthly_estimate.amount)
                .unwrap_or(0);
            Some(Money::new(max_cost - min_cost, crate::models::Currency::Usd))
        } else {
            None
        };

        Ok(vec![PriceComparison {
            comparison_key,
            provider_prices,
            cheapest,
            potential_savings,
        }])
    }

    /// Calculate monthly estimate for a price entry
    fn calculate_monthly_estimate(&self, price: &PriceEntry) -> Money {
        use crate::models::PricingUnit;

        let monthly_hours = 730; // ~30.4 days * 24 hours
        let amount = match price.pricing_unit {
            PricingUnit::Hour => price.unit_price.amount * monthly_hours,
            PricingUnit::Month => price.unit_price.amount,
            PricingUnit::Gb | PricingUnit::GbMonth => {
                // Assume 100 GB for estimation
                price.unit_price.amount * 100
            }
            PricingUnit::GbHour => {
                // Assume 100 GB for estimation
                price.unit_price.amount * 100 * monthly_hours
            }
            PricingUnit::Request | PricingUnit::MillionRequests => {
                // Assume 1M requests/month
                price.unit_price.amount
            }
            PricingUnit::Second => {
                // Convert to monthly (730 hours * 3600 seconds)
                price.unit_price.amount * monthly_hours * 3600
            }
            PricingUnit::Unit => price.unit_price.amount,
        };

        Money::new(amount, price.unit_price.currency)
    }

    /// Get count of all prices
    pub async fn count(&self) -> PricingResult<usize> {
        self.repository.count().await
    }

    /// Get count by provider
    pub async fn count_by_provider(&self, provider: CloudProvider) -> PricingResult<usize> {
        self.repository.count_by_provider(provider).await
    }

    /// Get available regions for a provider
    pub async fn get_regions(&self, provider: CloudProvider) -> PricingResult<Vec<String>> {
        self.repository.get_regions_for_provider(provider).await
    }

    /// Clean up expired price entries
    pub async fn cleanup_expired(&self) -> PricingResult<usize> {
        self.repository.delete_expired().await
    }
}
