//! Price Collector Service
//!
//! Orchestrates price collection from all configured cloud providers.

use chrono::{DateTime, Utc};
use domain_pricing::{CloudProvider, ResourceType};
use eyre::Result;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::Serialize;
use std::str::FromStr;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::providers::{
    AwsPricingProvider, AzurePricingProvider, GcpPricingProvider, ProviderRegistry,
};

/// Result of a price collection run
#[derive(Debug, Clone, Serialize)]
pub struct CollectionResult {
    pub prices_collected: usize,
    pub prices_updated: usize,
    pub errors: usize,
    pub providers_collected: Vec<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Status of the collector service
#[derive(Debug, Clone, Serialize)]
pub struct CollectorStatus {
    pub last_collection: Option<DateTime<Utc>>,
    pub last_result: Option<CollectionResult>,
    pub providers: Vec<ProviderStatus>,
    pub is_healthy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub provider: String,
    pub enabled: bool,
    pub is_healthy: bool,
    pub supported_regions: Vec<String>,
}

/// Main price collector service
pub struct PriceCollector {
    db: DatabaseConnection,
    config: Config,
    registry: ProviderRegistry,
}

impl PriceCollector {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        let mut registry = ProviderRegistry::new();

        // Register all providers
        registry.register(Box::new(AwsPricingProvider::new(config.aws.clone())));
        registry.register(Box::new(AzurePricingProvider::new(config.azure.clone())));
        registry.register(Box::new(GcpPricingProvider::new(config.gcp.clone())));

        Self {
            db,
            config,
            registry,
        }
    }

    /// Run a one-time collection
    pub async fn collect(
        &self,
        providers: Option<&[String]>,
        resource_types: Option<&[String]>,
        regions: Option<&[String]>,
        _force: bool,
    ) -> Result<CollectionResult> {
        let start = std::time::Instant::now();
        let mut total_collected = 0;
        let mut total_updated = 0;
        let mut total_errors = 0;
        let mut providers_collected = Vec::new();

        // Parse resource types
        let resource_type_filters: Vec<ResourceType> = resource_types
            .map(|types| {
                types
                    .iter()
                    .filter_map(|t| ResourceType::from_str(t).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse regions
        let region_filters: Vec<String> = regions
            .map(|r| r.to_vec())
            .unwrap_or_else(|| self.config.default_regions.clone());

        // Get providers to collect from
        let provider_filters: Option<Vec<CloudProvider>> = providers.map(|p| {
            p.iter()
                .filter_map(|name| match name.to_lowercase().as_str() {
                    "aws" => Some(CloudProvider::Aws),
                    "azure" => Some(CloudProvider::Azure),
                    "gcp" => Some(CloudProvider::Gcp),
                    _ => None,
                })
                .collect()
        });

        // Collect from each configured provider
        for provider in self.registry.get_configured_providers() {
            // Skip if provider filter is set and this provider is not in it
            if let Some(ref filters) = provider_filters {
                if !filters.contains(&provider.provider()) {
                    continue;
                }
            }

            info!(
                provider = provider.name(),
                "Starting price collection"
            );

            match provider
                .collect_prices(&resource_type_filters, &region_filters)
                .await
            {
                Ok(prices) => {
                    let count = prices.len();
                    info!(
                        provider = provider.name(),
                        count = count,
                        "Collected prices"
                    );

                    // Store prices in database
                    match self.store_prices(&prices).await {
                        Ok(updated) => {
                            total_collected += count;
                            total_updated += updated;
                            providers_collected.push(provider.name().to_string());
                        }
                        Err(e) => {
                            error!(
                                provider = provider.name(),
                                error = %e,
                                "Failed to store prices"
                            );
                            total_errors += 1;
                        }
                    }
                }
                Err(e) => {
                    error!(
                        provider = provider.name(),
                        error = %e,
                        "Failed to collect prices"
                    );
                    total_errors += 1;
                }
            }
        }

        let duration = start.elapsed();

        let result = CollectionResult {
            prices_collected: total_collected,
            prices_updated: total_updated,
            errors: total_errors,
            providers_collected,
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
        };

        // Record metrics
        observability::counter!("pricing_collection_total", "status" => "success")
            .increment(total_collected as u64);
        observability::counter!("pricing_collection_errors_total").increment(total_errors as u64);
        observability::histogram!("pricing_collection_duration_seconds")
            .record(duration.as_secs_f64());

        Ok(result)
    }

    /// Store prices in the database
    async fn store_prices(
        &self,
        prices: &[domain_pricing::CreatePriceEntry],
    ) -> Result<usize> {
        let mut updated = 0;

        for price in prices {
            let attributes_json = serde_json::to_string(&price.attributes)?;
            let instance_type_str = price.instance_type.as_deref().unwrap_or("");
            let expiration_date_str = price
                .expiration_date
                .map(|d| format!("'{}'", d.format("%Y-%m-%d %H:%M:%S")))
                .unwrap_or_else(|| "NULL".to_string());

            // Use raw SQL with SeaORM
            // Column names match the migration: unit_price_amount, unit_price_currency
            // gen_random_uuid() requires pgcrypto extension (installed in bootstrap migration)
            let sql = format!(
                r#"
                INSERT INTO cloud_prices (
                    id, provider, resource_type, sku, service_name, product_family,
                    instance_type, region, unit_price_amount, unit_price_currency, pricing_unit,
                    description, attributes, effective_date, expiration_date,
                    collected_at, created_at, updated_at
                )
                VALUES (gen_random_uuid(), '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', {}, NOW(), NOW(), NOW())
                ON CONFLICT (sku, provider, region)
                DO UPDATE SET
                    unit_price_amount = {},
                    description = '{}',
                    attributes = '{}',
                    effective_date = '{}',
                    collected_at = NOW(),
                    updated_at = NOW()
                "#,
                price.provider,
                price.resource_type,
                price.sku.replace("'", "''"),
                price.service_name.replace("'", "''"),
                price.product_family.replace("'", "''"),
                instance_type_str.replace("'", "''"),
                price.region.replace("'", "''"),
                price.unit_price.amount,
                price.unit_price.currency,
                price.pricing_unit,
                price.description.replace("'", "''"),
                attributes_json.replace("'", "''"),
                price.effective_date.format("%Y-%m-%d %H:%M:%S"),
                expiration_date_str,
                price.unit_price.amount,
                price.description.replace("'", "''"),
                attributes_json.replace("'", "''"),
                price.effective_date.format("%Y-%m-%d %H:%M:%S"),
            );

            // Log first SQL for debugging
            if updated == 0 {
                debug!(sql = %sql, "First INSERT SQL");
            }

            let stmt = Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql.clone());
            let result = self.db.execute_raw(stmt).await;

            match result {
                Ok(_) => updated += 1,
                Err(e) => {
                    error!(
                        sku = %price.sku,
                        error = %e,
                        sql = %sql,
                        "Failed to upsert price"
                    );
                }
            }
        }

        Ok(updated)
    }

    /// Run as a scheduled service
    pub async fn run_scheduled(&self, cron_expr: &str) -> Result<()> {
        info!(cron = cron_expr, "Starting scheduled price collection");

        let sched = JobScheduler::new().await?;

        // Clone what we need for the job closure
        let db = self.db.clone();
        let config = self.config.clone();

        let job = Job::new_async(cron_expr, move |_uuid, _l| {
            let db = db.clone();
            let config = config.clone();

            Box::pin(async move {
                info!("Running scheduled price collection");

                let collector = PriceCollector::new(db, config);
                match collector.collect(None, None, None, false).await {
                    Ok(result) => {
                        info!(
                            collected = result.prices_collected,
                            updated = result.prices_updated,
                            errors = result.errors,
                            "Scheduled collection complete"
                        );
                    }
                    Err(e) => {
                        error!(error = %e, "Scheduled collection failed");
                    }
                }
            })
        })?;

        sched.add(job).await?;
        sched.start().await?;

        // Keep running until interrupted
        info!("Scheduler started, waiting for jobs...");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Get collector status
    pub async fn get_status(&self) -> Result<CollectorStatus> {
        let mut provider_statuses = Vec::new();

        for provider in self.registry.all_providers() {
            let is_healthy = provider.health_check().await.unwrap_or(false);

            provider_statuses.push(ProviderStatus {
                name: provider.name().to_string(),
                provider: provider.provider().to_string(),
                enabled: provider.is_configured(),
                is_healthy,
                supported_regions: provider.supported_regions(),
            });
        }

        let is_healthy = provider_statuses.iter().any(|p| p.enabled && p.is_healthy);

        Ok(CollectorStatus {
            last_collection: None, // Would query from database
            last_result: None,
            providers: provider_statuses,
            is_healthy,
        })
    }
}
