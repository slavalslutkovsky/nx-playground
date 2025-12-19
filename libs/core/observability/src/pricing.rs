//! Pricing-specific metrics for cloud cost optimization.

use metrics::{counter, gauge, histogram};
use std::time::Instant;

/// Pricing metrics recorder
pub struct PricingMetrics;

impl PricingMetrics {
    // =========================================================================
    // Operation Metrics
    // =========================================================================

    /// Record a list prices operation
    pub fn record_list_prices(provider: &str, count: usize, duration_ms: u64) {
        counter!("pricing_operations_total", "operation" => "list", "provider" => provider.to_string())
            .increment(1);
        histogram!("pricing_operation_duration_seconds", "operation" => "list", "provider" => provider.to_string())
            .record(duration_ms as f64 / 1000.0);

        tracing::debug!(
            provider = provider,
            count = count,
            duration_ms = duration_ms,
            "Listed prices"
        );
    }

    /// Record a price creation
    pub fn record_price_created(provider: &str, resource_type: &str) {
        counter!(
            "pricing_operations_total",
            "operation" => "create",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string()
        )
        .increment(1);
    }

    /// Record a price update
    pub fn record_price_updated(provider: &str, resource_type: &str) {
        counter!(
            "pricing_operations_total",
            "operation" => "update",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string()
        )
        .increment(1);
    }

    /// Record a price deletion
    pub fn record_price_deleted(provider: &str) {
        counter!("pricing_operations_total", "operation" => "delete", "provider" => provider.to_string())
            .increment(1);
    }

    /// Record a price comparison operation
    pub fn record_price_comparison(providers_count: usize, results_count: usize, duration_ms: u64) {
        counter!("pricing_comparisons_total").increment(1);
        histogram!("pricing_operation_duration_seconds", "operation" => "compare")
            .record(duration_ms as f64 / 1000.0);

        tracing::debug!(
            providers = providers_count,
            results = results_count,
            duration_ms = duration_ms,
            "Compared prices"
        );
    }

    // =========================================================================
    // Gauge Metrics (Current State)
    // =========================================================================

    /// Set the total pricing entries count by provider
    pub fn set_pricing_entries_count(provider: &str, count: usize) {
        gauge!("pricing_entries_total", "provider" => provider.to_string()).set(count as f64);
    }

    /// Set the pricing entries count by resource type
    pub fn set_pricing_entries_by_type(provider: &str, resource_type: &str, count: usize) {
        gauge!(
            "pricing_entries_total",
            "provider" => provider.to_string(),
            "resource_type" => resource_type.to_string()
        )
        .set(count as f64);
    }

    // =========================================================================
    // Collection Metrics
    // =========================================================================

    /// Record a collection job start
    pub fn record_collection_started(provider: &str, job_type: &str) {
        counter!(
            "price_collection_jobs_total",
            "provider" => provider.to_string(),
            "job_type" => job_type.to_string(),
            "status" => "started"
        )
        .increment(1);
    }

    /// Record a collection job completion
    pub fn record_collection_completed(
        provider: &str,
        job_type: &str,
        prices_collected: usize,
        duration_secs: f64,
    ) {
        counter!(
            "price_collection_jobs_total",
            "provider" => provider.to_string(),
            "job_type" => job_type.to_string(),
            "status" => "completed"
        )
        .increment(1);

        histogram!(
            "price_collection_duration_seconds",
            "provider" => provider.to_string(),
            "job_type" => job_type.to_string()
        )
        .record(duration_secs);

        gauge!("prices_collected_last_run", "provider" => provider.to_string())
            .set(prices_collected as f64);

        tracing::info!(
            provider = provider,
            job_type = job_type,
            prices_collected = prices_collected,
            duration_secs = duration_secs,
            "Collection job completed"
        );
    }

    /// Record a collection job failure
    pub fn record_collection_failed(provider: &str, job_type: &str, error: &str) {
        counter!(
            "price_collection_jobs_total",
            "provider" => provider.to_string(),
            "job_type" => job_type.to_string(),
            "status" => "failed"
        )
        .increment(1);

        tracing::error!(
            provider = provider,
            job_type = job_type,
            error = error,
            "Collection job failed"
        );
    }

    // =========================================================================
    // Recommendation Metrics
    // =========================================================================

    /// Set the total recommendations count
    pub fn set_recommendations_count(
        recommendation_type: &str,
        status: &str,
        count: usize,
    ) {
        gauge!(
            "recommendations_total",
            "type" => recommendation_type.to_string(),
            "status" => status.to_string()
        )
        .set(count as f64);
    }

    /// Set the total potential savings
    pub fn set_potential_savings(provider: &str, amount_cents: i64) {
        gauge!("potential_savings_usd", "provider" => provider.to_string())
            .set(amount_cents as f64);
    }

    /// Record a recommendation being applied
    pub fn record_recommendation_applied(recommendation_type: &str, savings_cents: i64) {
        counter!(
            "recommendations_applied_total",
            "type" => recommendation_type.to_string()
        )
        .increment(1);

        tracing::info!(
            recommendation_type = recommendation_type,
            savings_cents = savings_cents,
            "Recommendation applied"
        );
    }
}

/// Timer guard for automatic duration recording.
///
/// Records the duration when `stop()` is called or when dropped.
pub struct PricingTimer {
    start: Instant,
    operation: String,
    provider: String,
    stopped: bool,
}

impl PricingTimer {
    /// Start a new timer for an operation
    pub fn new(operation: &str, provider: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
            provider: provider.to_string(),
            stopped: false,
        }
    }

    /// Stop the timer and record the duration. Returns duration in milliseconds.
    pub fn stop(&mut self) -> u64 {
        if self.stopped {
            return 0;
        }
        self.stopped = true;

        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        histogram!(
            "pricing_operation_duration_seconds",
            "operation" => self.operation.clone(),
            "provider" => self.provider.clone()
        )
        .record(duration.as_secs_f64());

        duration_ms
    }
}

impl Drop for PricingTimer {
    fn drop(&mut self) {
        // Record on drop if not explicitly stopped
        if !self.stopped {
            self.stop();
        }
    }
}
