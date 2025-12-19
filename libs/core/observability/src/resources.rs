//! Resource-specific metrics for cloud infrastructure.

use metrics::{counter, gauge, histogram};

/// Resource metrics recorder
pub struct ResourceMetrics;

impl ResourceMetrics {
    // =========================================================================
    // Resource Inventory Metrics
    // =========================================================================

    /// Set the total resources count by provider and type
    pub fn set_resources_count(provider: &str, resource_type: &str, count: usize) {
        gauge!(
            "cloud_resources_total",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string()
        )
        .set(count as f64);
    }

    /// Set the total resources count by provider
    pub fn set_provider_resources_count(provider: &str, count: usize) {
        gauge!("cloud_resources_total", "provider" => provider.to_string()).set(count as f64);
    }

    /// Set the total resources count by region
    pub fn set_region_resources_count(provider: &str, region: &str, count: usize) {
        gauge!(
            "cloud_resources_total",
            "provider" => provider.to_string(),
            "region" => region.to_string()
        )
        .set(count as f64);
    }

    // =========================================================================
    // Sync Metrics
    // =========================================================================

    /// Record a resource sync operation start
    pub fn record_sync_started(provider: &str) {
        counter!(
            "resource_sync_operations_total",
            "provider" => provider.to_string(),
            "status" => "started"
        )
        .increment(1);

        tracing::debug!(provider = provider, "Resource sync started");
    }

    /// Record a resource sync operation success
    pub fn record_sync_completed(
        provider: &str,
        resources_synced: usize,
        duration_secs: f64,
    ) {
        counter!(
            "resource_sync_operations_total",
            "provider" => provider.to_string(),
            "status" => "completed"
        )
        .increment(1);

        histogram!(
            "resource_sync_duration_seconds",
            "provider" => provider.to_string()
        )
        .record(duration_secs);

        tracing::info!(
            provider = provider,
            resources_synced = resources_synced,
            duration_secs = duration_secs,
            "Resource sync completed"
        );
    }

    /// Record a resource sync operation failure
    pub fn record_sync_failed(provider: &str, error: &str) {
        counter!(
            "resource_sync_operations_total",
            "provider" => provider.to_string(),
            "status" => "failed"
        )
        .increment(1);

        tracing::error!(
            provider = provider,
            error = error,
            "Resource sync failed"
        );
    }

    // =========================================================================
    // Resource Change Metrics
    // =========================================================================

    /// Record a resource being created
    pub fn record_resource_created(provider: &str, resource_type: &str) {
        counter!(
            "resource_changes_total",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string(),
            "change_type" => "created"
        )
        .increment(1);
    }

    /// Record a resource being updated
    pub fn record_resource_updated(provider: &str, resource_type: &str) {
        counter!(
            "resource_changes_total",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string(),
            "change_type" => "updated"
        )
        .increment(1);
    }

    /// Record a resource being deleted
    pub fn record_resource_deleted(provider: &str, resource_type: &str) {
        counter!(
            "resource_changes_total",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string(),
            "change_type" => "deleted"
        )
        .increment(1);
    }

    // =========================================================================
    // Cost Metrics
    // =========================================================================

    /// Set the estimated monthly cost by provider
    pub fn set_estimated_monthly_cost(provider: &str, cost_cents: i64) {
        gauge!(
            "estimated_monthly_cost_usd",
            "provider" => provider.to_string()
        )
        .set(cost_cents as f64 / 100.0);
    }

    /// Set the estimated monthly cost by resource type
    pub fn set_resource_type_cost(
        provider: &str,
        resource_type: &str,
        cost_cents: i64,
    ) {
        gauge!(
            "estimated_monthly_cost_usd",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string()
        )
        .set(cost_cents as f64 / 100.0);
    }
}
