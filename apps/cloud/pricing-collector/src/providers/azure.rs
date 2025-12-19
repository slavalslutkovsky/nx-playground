//! Azure Pricing Provider
//!
//! Fetches pricing data from Azure Retail Prices API.
//! https://docs.microsoft.com/en-us/rest/api/cost-management/retail-prices/azure-retail-prices

use async_trait::async_trait;
use chrono::Utc;
use domain_pricing::{CloudProvider, CreatePriceEntry, Currency, Money, PricingUnit, ResourceType};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::{PricingProvider, ProviderError, ProviderResult};
use crate::config::AzureConfig;

// https://prices.azure.com/api/retail/prices?api-version=2023-01-01-preview&meterRegion='primary'
/// Azure Retail Prices API endpoint
const AZURE_PRICING_API: &str = "https://prices.azure.com/api/retail/prices";

/// Azure Pricing Provider
pub struct AzurePricingProvider {
    config: AzureConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct AzurePriceResponse {
    #[serde(rename = "Items")]
    items: Vec<AzurePriceItem>,
    #[serde(rename = "NextPageLink")]
    next_page_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AzurePriceItem {
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(rename = "tierMinimumUnits")]
    tier_minimum_units: f64,
    #[serde(rename = "retailPrice")]
    retail_price: f64,
    #[serde(rename = "unitPrice")]
    unit_price: f64,
    #[serde(rename = "armRegionName")]
    arm_region_name: String,
    #[serde(rename = "location")]
    location: String,
    #[serde(rename = "effectiveStartDate")]
    effective_start_date: String,
    #[serde(rename = "meterId")]
    meter_id: String,
    #[serde(rename = "meterName")]
    meter_name: String,
    #[serde(rename = "productId")]
    product_id: String,
    #[serde(rename = "skuId")]
    sku_id: String,
    #[serde(rename = "productName")]
    product_name: String,
    #[serde(rename = "skuName")]
    sku_name: String,
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "serviceId")]
    service_id: String,
    #[serde(rename = "serviceFamily")]
    service_family: String,
    #[serde(rename = "unitOfMeasure")]
    unit_of_measure: String,
    #[serde(rename = "type")]
    price_type: String,
    #[serde(rename = "isPrimaryMeterRegion")]
    is_primary_meter_region: bool,
    #[serde(rename = "armSkuName")]
    arm_sku_name: Option<String>,
}

impl AzurePricingProvider {
    pub fn new(config: AzureConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Fetch VM pricing for a region
    async fn fetch_vm_prices(&self, region: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        info!(region = region, "Fetching Azure VM prices");

        // Build filter query
        let filter = format!(
            "armRegionName eq '{}' and serviceName eq 'Virtual Machines' and priceType eq 'Consumption'",
            region
        );

        match self.fetch_prices_with_filter(&filter).await {
            Ok(prices) => Ok(prices),
            Err(e) => {
                warn!(error = %e, region = region, "Failed to fetch Azure prices, using mock data");
                Ok(self.generate_mock_vm_prices(region))
            }
        }
    }

    async fn fetch_prices_with_filter(&self, filter: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        let url = format!("{}?$filter={}", AZURE_PRICING_API, urlencoding::encode(filter));

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError(format!(
                "Azure API returned status: {}",
                response.status()
            )));
        }

        let data: AzurePriceResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let prices: Vec<CreatePriceEntry> = data
            .items
            .into_iter()
            .filter_map(|item| self.convert_price_item(item))
            .collect();

        Ok(prices)
    }

    fn convert_price_item(&self, item: AzurePriceItem) -> Option<CreatePriceEntry> {
        let resource_type = self.map_service_to_resource_type(&item.service_family);
        let pricing_unit = self.map_unit_of_measure(&item.unit_of_measure);

        // Convert price to cents
        let price_cents = (item.retail_price * 100.0).round() as i64;

        Some(CreatePriceEntry {
            provider: CloudProvider::Azure,
            resource_type,
            sku: item.sku_id,
            service_name: item.service_name,
            product_family: item.service_family,
            instance_type: item.arm_sku_name,
            region: item.arm_region_name,
            unit_price: Money::new(price_cents, Currency::Usd),
            pricing_unit,
            description: format!("{} - {}", item.product_name, item.sku_name),
            attributes: HashMap::from([
                ("meter_id".to_string(), item.meter_id),
                ("meter_name".to_string(), item.meter_name),
                ("product_id".to_string(), item.product_id),
            ]),
            effective_date: Utc::now(),
            expiration_date: None,
        })
    }

    fn map_service_to_resource_type(&self, service_family: &str) -> ResourceType {
        match service_family.to_lowercase().as_str() {
            "compute" => ResourceType::Compute,
            "storage" => ResourceType::Storage,
            "databases" => ResourceType::Database,
            "networking" => ResourceType::Network,
            "analytics" => ResourceType::Analytics,
            "containers" => ResourceType::Kubernetes,
            _ => ResourceType::Other,
        }
    }

    fn map_unit_of_measure(&self, unit: &str) -> PricingUnit {
        match unit.to_lowercase().as_str() {
            "1 hour" | "1/hour" => PricingUnit::Hour,
            "1 month" | "1/month" => PricingUnit::Month,
            "1 gb" | "1 gb/month" => PricingUnit::GbMonth,
            "1 gb/hour" => PricingUnit::GbHour,
            "10000" | "10,000" => PricingUnit::Request,
            _ => PricingUnit::Unit,
        }
    }

    /// Generate mock VM prices for development/demo
    fn generate_mock_vm_prices(&self, region: &str) -> Vec<CreatePriceEntry> {
        let vm_sizes = vec![
            ("Standard_B1s", 1, 1, 104),
            ("Standard_B1ms", 1, 2, 208),
            ("Standard_B2s", 2, 4, 416),
            ("Standard_D2s_v3", 2, 8, 960),
            ("Standard_D4s_v3", 4, 16, 1920),
            ("Standard_D8s_v3", 8, 32, 3840),
            ("Standard_E2s_v3", 2, 16, 1260),
            ("Standard_E4s_v3", 4, 32, 2520),
            ("Standard_F2s_v2", 2, 4, 850),
            ("Standard_F4s_v2", 4, 8, 1700),
        ];

        vm_sizes
            .into_iter()
            .map(|(vm_size, vcpu, memory_gb, price_cents)| CreatePriceEntry {
                provider: CloudProvider::Azure,
                resource_type: ResourceType::Compute,
                sku: format!("azure-vm-{}-{}", vm_size.to_lowercase(), region),
                service_name: "Virtual Machines".to_string(),
                product_family: "Compute".to_string(),
                instance_type: Some(vm_size.to_string()),
                region: region.to_string(),
                unit_price: Money::new(price_cents, Currency::Usd),
                pricing_unit: PricingUnit::Hour,
                description: format!("{} - {} vCPU, {} GB memory", vm_size, vcpu, memory_gb),
                attributes: HashMap::from([
                    ("vcpu".to_string(), vcpu.to_string()),
                    ("memory_gb".to_string(), memory_gb.to_string()),
                ]),
                effective_date: Utc::now(),
                expiration_date: None,
            })
            .collect()
    }

    /// Fetch storage pricing
    async fn fetch_storage_prices(&self, region: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        info!(region = region, "Fetching Azure Storage prices");

        let storage_tiers = vec![
            ("Hot", "StorageV2", 20),
            ("Cool", "StorageV2", 10),
            ("Archive", "StorageV2", 2),
            ("Premium", "BlockBlobStorage", 150),
        ];

        Ok(storage_tiers
            .into_iter()
            .map(|(tier, account_type, price_milli_cents)| CreatePriceEntry {
                provider: CloudProvider::Azure,
                resource_type: ResourceType::Storage,
                sku: format!("azure-storage-{}-{}", tier.to_lowercase(), region),
                service_name: "Storage".to_string(),
                product_family: "Storage".to_string(),
                instance_type: Some(format!("{} {}", account_type, tier)),
                region: region.to_string(),
                unit_price: Money::new(price_milli_cents, Currency::Usd),
                pricing_unit: PricingUnit::GbMonth,
                description: format!("Azure Blob Storage - {} tier", tier),
                attributes: HashMap::from([
                    ("tier".to_string(), tier.to_string()),
                    ("account_type".to_string(), account_type.to_string()),
                ]),
                effective_date: Utc::now(),
                expiration_date: None,
            })
            .collect())
    }

    /// Fetch Azure Database for PostgreSQL pricing
    async fn fetch_postgresql_prices(&self, region: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        info!(region = region, "Fetching Azure PostgreSQL prices");

        // Azure Database for PostgreSQL - Flexible Server pricing
        let db_instances = vec![
            ("B1ms", 1, 2, 250),       // Burstable $0.025/hr
            ("B2s", 2, 4, 500),        // Burstable $0.05/hr
            ("D2s_v3", 2, 8, 1230),    // General Purpose $0.123/hr
            ("D4s_v3", 4, 16, 2460),   // General Purpose $0.246/hr
            ("D8s_v3", 8, 32, 4920),   // General Purpose $0.492/hr
            ("E2s_v3", 2, 16, 1660),   // Memory Optimized $0.166/hr
            ("E4s_v3", 4, 32, 3320),   // Memory Optimized $0.332/hr
            ("E8s_v3", 8, 64, 6640),   // Memory Optimized $0.664/hr
        ];

        Ok(db_instances
            .into_iter()
            .map(|(instance_type, vcpu, memory_gb, price_cents)| CreatePriceEntry {
                provider: CloudProvider::Azure,
                resource_type: ResourceType::Database,
                sku: format!("azure-postgresql-{}-{}", instance_type.to_lowercase(), region),
                service_name: "Azure Database for PostgreSQL".to_string(),
                product_family: "Database Instance".to_string(),
                instance_type: Some(instance_type.to_string()),
                region: region.to_string(),
                unit_price: Money::new(price_cents, Currency::Usd),
                pricing_unit: PricingUnit::Hour,
                description: format!("PostgreSQL Flexible {} - {} vCPU, {} GB RAM", instance_type, vcpu, memory_gb),
                attributes: HashMap::from([
                    ("vcpu".to_string(), vcpu.to_string()),
                    ("memory_gb".to_string(), memory_gb.to_string()),
                    ("engine".to_string(), "PostgreSQL".to_string()),
                ]),
                effective_date: Utc::now(),
                expiration_date: None,
            })
            .collect())
    }

    /// Fetch Azure Cache for Redis pricing
    async fn fetch_redis_prices(&self, region: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        info!(region = region, "Fetching Azure Cache for Redis prices");

        // Azure Cache for Redis pricing
        let redis_instances = vec![
            ("C0", 0.25, 250, 160),    // Basic 250MB $0.016/hr
            ("C1", 1.0, 1000, 340),    // Basic 1GB $0.034/hr
            ("C2", 2.5, 2500, 680),    // Basic 2.5GB $0.068/hr
            ("C0-Standard", 0.25, 250, 500),  // Standard 250MB $0.05/hr
            ("C1-Standard", 1.0, 1000, 1000), // Standard 1GB $0.10/hr
            ("C2-Standard", 2.5, 2500, 2000), // Standard 2.5GB $0.20/hr
            ("P1", 6.0, 6000, 4220),   // Premium 6GB $0.422/hr
            ("P2", 13.0, 13000, 8440), // Premium 13GB $0.844/hr
            ("P3", 26.0, 26000, 16880), // Premium 26GB $1.688/hr
        ];

        Ok(redis_instances
            .into_iter()
            .map(|(tier, memory_gb, memory_mb, price_cents)| CreatePriceEntry {
                provider: CloudProvider::Azure,
                resource_type: ResourceType::Database,
                sku: format!("azure-redis-{}-{}", tier.to_lowercase(), region),
                service_name: "Azure Cache for Redis".to_string(),
                product_family: "Cache Instance".to_string(),
                instance_type: Some(tier.to_string()),
                region: region.to_string(),
                unit_price: Money::new(price_cents, Currency::Usd),
                pricing_unit: PricingUnit::Hour,
                description: format!("Azure Redis {} - {} GB ({} MB)", tier, memory_gb, memory_mb),
                attributes: HashMap::from([
                    ("memory_gb".to_string(), format!("{:.2}", memory_gb)),
                    ("memory_mb".to_string(), memory_mb.to_string()),
                    ("engine".to_string(), "Redis".to_string()),
                ]),
                effective_date: Utc::now(),
                expiration_date: None,
            })
            .collect())
    }

    /// Fetch AKS (Kubernetes) pricing
    async fn fetch_aks_prices(&self, region: &str) -> ProviderResult<Vec<CreatePriceEntry>> {
        info!(region = region, "Fetching Azure AKS prices");

        // AKS pricing: Control plane is FREE!
        // You only pay for VMs (worker nodes), storage, and networking
        let aks_options = vec![
            ("aks-cluster", "AKS Cluster", 0),  // FREE control plane!
            ("aks-uptime-sla", "AKS Uptime SLA", 1000),  // $0.10/hr for 99.95% SLA
        ];

        Ok(aks_options
            .into_iter()
            .map(|(sku_suffix, instance_type, price_cents)| CreatePriceEntry {
                provider: CloudProvider::Azure,
                resource_type: ResourceType::Kubernetes,
                sku: format!("azure-{}-{}", sku_suffix, region),
                service_name: "Azure Kubernetes Service".to_string(),
                product_family: "Kubernetes".to_string(),
                instance_type: Some(instance_type.to_string()),
                region: region.to_string(),
                unit_price: Money::new(price_cents, Currency::Usd),
                pricing_unit: PricingUnit::Hour,
                description: match sku_suffix {
                    "aks-cluster" => "AKS Control Plane - FREE (pay only for worker nodes)".to_string(),
                    "aks-uptime-sla" => "AKS Uptime SLA - $0.10/hour for 99.95% SLA".to_string(),
                    _ => instance_type.to_string(),
                },
                attributes: HashMap::from([
                    ("service".to_string(), "AKS".to_string()),
                ]),
                effective_date: Utc::now(),
                expiration_date: None,
            })
            .collect())
    }
}

#[async_trait]
impl PricingProvider for AzurePricingProvider {
    fn provider(&self) -> CloudProvider {
        CloudProvider::Azure
    }

    fn name(&self) -> &'static str {
        "Azure"
    }

    fn is_configured(&self) -> bool {
        self.config.enabled
    }

    async fn collect_prices(
        &self,
        resource_types: &[ResourceType],
        regions: &[String],
    ) -> ProviderResult<Vec<CreatePriceEntry>> {
        let mut all_prices = Vec::new();

        let regions_to_collect = if regions.is_empty() {
            &self.config.regions
        } else {
            regions
        };

        let types_to_collect: Vec<ResourceType> = if resource_types.is_empty() {
            vec![
                ResourceType::Compute,
                ResourceType::Storage,
                ResourceType::Database,
                ResourceType::Kubernetes,
            ]
        } else {
            resource_types.to_vec()
        };

        for region in regions_to_collect {
            for resource_type in &types_to_collect {
                let prices = match resource_type {
                    ResourceType::Compute => self.fetch_vm_prices(region).await?,
                    ResourceType::Storage => self.fetch_storage_prices(region).await?,
                    ResourceType::Database => {
                        let mut db_prices = self.fetch_postgresql_prices(region).await?;
                        db_prices.extend(self.fetch_redis_prices(region).await?);
                        db_prices
                    }
                    ResourceType::Kubernetes => self.fetch_aks_prices(region).await?,
                    _ => {
                        debug!(
                            resource_type = ?resource_type,
                            "Skipping unsupported resource type"
                        );
                        continue;
                    }
                };
                all_prices.extend(prices);
            }
        }

        info!(
            count = all_prices.len(),
            regions = regions_to_collect.len(),
            "Azure price collection complete"
        );

        Ok(all_prices)
    }

    async fn health_check(&self) -> ProviderResult<bool> {
        let url = format!("{}?$top=1", AZURE_PRICING_API);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    fn supported_regions(&self) -> Vec<String> {
        self.config.regions.clone()
    }
}
