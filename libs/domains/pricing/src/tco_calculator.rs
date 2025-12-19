//! TCO (Total Cost of Ownership) Calculator
//!
//! Calculates and compares costs between managed cloud services
//! and self-managed CNCF tools running on Kubernetes.

use crate::cncf_models::{
    get_cncf_tools, CncfTool, CostRecommendation, DeploymentMode, InfrastructureCostComparison,
    ResourceRequirements, TcoCalculationRequest, TcoCalculationResult,
};
use crate::models::{CloudProvider, Currency, Money, PriceFilter, ResourceType};
use crate::repository::PricingRepository;
use crate::service::PricingService;
use crate::PricingResult;
use std::collections::HashMap;
use std::sync::Arc;

/// TCO Calculator for comparing managed vs self-managed costs
pub struct TcoCalculator<R: PricingRepository> {
    service: Arc<PricingService<R>>,
    cncf_tools: Vec<CncfTool>,
}

impl<R: PricingRepository> TcoCalculator<R> {
    pub fn new(service: Arc<PricingService<R>>) -> Self {
        Self {
            service,
            cncf_tools: get_cncf_tools(),
        }
    }

    /// Get all available CNCF tools
    pub fn get_cncf_tools(&self) -> &[CncfTool] {
        &self.cncf_tools
    }

    /// Get a specific CNCF tool by ID
    pub fn get_tool(&self, tool_id: &str) -> Option<&CncfTool> {
        self.cncf_tools.iter().find(|t| t.id == tool_id)
    }

    /// Calculate TCO for a specific tool and deployment mode
    pub async fn calculate_tco(
        &self,
        request: TcoCalculationRequest,
    ) -> PricingResult<TcoCalculationResult> {
        let tool = self
            .get_tool(&request.tool_id)
            .ok_or_else(|| crate::error::PricingError::NotFound(request.tool_id.clone()))?
            .clone();

        // Get resource requirements based on deployment mode
        let workload_reqs = match request.deployment_mode {
            DeploymentMode::Minimal => &tool.minimal_requirements,
            DeploymentMode::HighAvailability => &tool.ha_requirements,
            DeploymentMode::Production => &tool.production_requirements,
        };

        // Calculate infrastructure costs
        let control_plane_cost = if request.include_control_plane {
            self.get_control_plane_cost(&request.provider, &request.region)
                .await?
        } else {
            Money::new(0, Currency::Usd)
        };

        let operator_compute_cost = self
            .calculate_compute_cost(
                &tool.operator_requirements,
                &request.provider,
                &request.region,
            )
            .await?;

        let workload_compute_cost = self
            .calculate_compute_cost(workload_reqs, &request.provider, &request.region)
            .await?;

        let storage_cost = self
            .calculate_storage_cost(workload_reqs, &request.provider, &request.region)
            .await?;

        // Estimate backup storage at 50% of primary storage cost
        let backup_storage_cost = Money::new(storage_cost.amount / 2, storage_cost.currency);

        let total_infra_cost = Money::new(
            control_plane_cost.amount
                + operator_compute_cost.amount
                + workload_compute_cost.amount
                + storage_cost.amount
                + backup_storage_cost.amount,
            Currency::Usd,
        );

        // Calculate operational costs
        let ops_hours = match request.deployment_mode {
            DeploymentMode::Minimal => tool.ops_hours.minimal_monthly_hours,
            DeploymentMode::HighAvailability => tool.ops_hours.ha_monthly_hours,
            DeploymentMode::Production => tool.ops_hours.production_monthly_hours,
        };

        let ops_cost = Money::new(
            (ops_hours * request.engineer_hourly_rate.amount as f32) as i64,
            request.engineer_hourly_rate.currency,
        );

        let amortized_ops_cost = Money::new(
            ops_cost.amount / request.workload_count.max(1) as i64,
            ops_cost.currency,
        );

        let total_self_managed_cost = Money::new(
            total_infra_cost.amount + amortized_ops_cost.amount,
            Currency::Usd,
        );

        // Get managed service cost for comparison
        let (managed_service_name, managed_service_sku, managed_service_cost) = self
            .get_managed_service_cost(&tool, &request.deployment_mode, &request.provider, &request.region)
            .await?;

        // Calculate savings
        let savings = managed_service_cost.amount - total_self_managed_cost.amount;
        let percentage_diff = if managed_service_cost.amount > 0 {
            (savings as f64 / managed_service_cost.amount as f64) * 100.0
        } else {
            0.0
        };

        // Calculate break-even ops hours
        let infra_savings = managed_service_cost.amount - total_infra_cost.amount;
        let break_even_hours = if request.engineer_hourly_rate.amount > 0 && infra_savings > 0 {
            infra_savings as f32 / request.engineer_hourly_rate.amount as f32
        } else {
            0.0
        };

        // Determine recommendation
        let recommendation = match percentage_diff {
            p if p > 30.0 => CostRecommendation::StronglySelfManaged,
            p if p > 10.0 => CostRecommendation::ConsiderSelfManaged,
            p if p > -10.0 => CostRecommendation::Similar,
            p if p > -30.0 => CostRecommendation::ConsiderManaged,
            _ => CostRecommendation::StronglyManaged,
        };

        Ok(TcoCalculationResult {
            tool_id: tool.id,
            tool_name: tool.name,
            deployment_mode: request.deployment_mode,
            provider: request.provider,
            region: request.region,
            control_plane_cost,
            operator_compute_cost,
            workload_compute_cost,
            storage_cost,
            backup_storage_cost,
            total_infra_cost,
            ops_hours_per_month: ops_hours,
            ops_cost,
            amortized_ops_cost,
            total_self_managed_cost,
            managed_service_name,
            managed_service_sku,
            managed_service_cost,
            savings_vs_managed: Money::new(savings, Currency::Usd),
            percentage_difference: percentage_diff,
            break_even_ops_hours: break_even_hours,
            recommendation,
        })
    }

    /// Compare all tools for a given provider/region
    pub async fn compare_infrastructure(
        &self,
        provider: CloudProvider,
        region: &str,
        deployment_mode: DeploymentMode,
        engineer_hourly_rate: Money,
        workload_count: i32,
    ) -> PricingResult<InfrastructureCostComparison> {
        let mut tool_comparisons = Vec::new();
        let mut recommendations = HashMap::new();
        let mut all_managed_cost = 0i64;
        let mut all_self_managed_cost = 0i64;
        let mut hybrid_cost = 0i64;

        for tool in &self.cncf_tools {
            let request = TcoCalculationRequest {
                tool_id: tool.id.clone(),
                deployment_mode,
                provider,
                region: region.to_string(),
                engineer_hourly_rate,
                include_control_plane: tool_comparisons.is_empty(), // Only first tool pays for control plane
                workload_count,
            };

            if let Ok(result) = self.calculate_tco(request).await {
                all_managed_cost += result.managed_service_cost.amount;
                all_self_managed_cost += result.total_self_managed_cost.amount;

                // Hybrid picks the cheaper option for each tool
                match result.recommendation {
                    CostRecommendation::StronglySelfManaged
                    | CostRecommendation::ConsiderSelfManaged => {
                        hybrid_cost += result.total_self_managed_cost.amount;
                    }
                    _ => {
                        hybrid_cost += result.managed_service_cost.amount;
                    }
                }

                recommendations.insert(tool.id.clone(), result.recommendation);
                tool_comparisons.push(result);
            }
        }

        Ok(InfrastructureCostComparison {
            all_managed_cost: Money::new(all_managed_cost, Currency::Usd),
            all_self_managed_cost: Money::new(all_self_managed_cost, Currency::Usd),
            hybrid_cost: Money::new(hybrid_cost, Currency::Usd),
            tool_comparisons,
            recommendations,
        })
    }

    /// Get K8s control plane cost for a provider
    async fn get_control_plane_cost(
        &self,
        provider: &CloudProvider,
        region: &str,
    ) -> PricingResult<Money> {
        // Try to get from database first
        let filter = PriceFilter {
            provider: Some(*provider),
            resource_type: Some(ResourceType::Kubernetes),
            regions: Some(region.to_string()),
            ..Default::default()
        };

        if let Ok(prices) = self.service.list(filter).await {
            // Look for cluster/control plane pricing
            if let Some(price) = prices.iter().find(|p| {
                p.instance_type
                    .as_ref()
                    .map(|t| t.to_lowercase().contains("cluster"))
                    .unwrap_or(false)
            }) {
                // Convert hourly to monthly (730 hours)
                return Ok(Money::new(price.unit_price.amount * 730, Currency::Usd));
            }
        }

        // Fallback to known pricing
        let monthly_cost = match provider {
            CloudProvider::Aws => 7300,   // $73/mo (EKS $0.10/hr)
            CloudProvider::Gcp => 7300,   // $73/mo (GKE Standard $0.10/hr)
            CloudProvider::Azure => 0,    // AKS is free!
        };

        Ok(Money::new(monthly_cost, Currency::Usd))
    }

    /// Calculate compute cost for given resource requirements
    async fn calculate_compute_cost(
        &self,
        reqs: &ResourceRequirements,
        provider: &CloudProvider,
        region: &str,
    ) -> PricingResult<Money> {
        // Find a matching compute instance
        let filter = PriceFilter {
            provider: Some(*provider),
            resource_type: Some(ResourceType::Compute),
            regions: Some(region.to_string()),
            ..Default::default()
        };

        let prices = self.service.list(filter).await.unwrap_or_default();

        // Find the cheapest instance that meets requirements
        let total_cpu = reqs.total_cpu_millicores() as f64 / 1000.0;
        let total_memory_gb = reqs.total_memory_mb() as f64 / 1024.0;

        let matching_price = prices.iter().find(|p| {
            let vcpu: f64 = p
                .attributes
                .get("vcpu")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            let memory: f64 = p
                .attributes
                .get("memory_gb")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            vcpu >= total_cpu && memory >= total_memory_gb
        });

        if let Some(price) = matching_price {
            // Convert hourly to monthly
            return Ok(Money::new(price.unit_price.amount * 730, Currency::Usd));
        }

        // Fallback: estimate based on resource requirements
        // Rough estimate: $0.05 per vCPU-hour, $0.007 per GB-hour
        let cpu_cost = (total_cpu * 0.05 * 730.0 * 100.0) as i64; // cents
        let mem_cost = (total_memory_gb * 0.007 * 730.0 * 100.0) as i64;

        Ok(Money::new(cpu_cost + mem_cost, Currency::Usd))
    }

    /// Calculate storage cost for given requirements
    async fn calculate_storage_cost(
        &self,
        reqs: &ResourceRequirements,
        provider: &CloudProvider,
        region: &str,
    ) -> PricingResult<Money> {
        let total_storage_gb = reqs.total_storage_gb();
        if total_storage_gb == 0 {
            return Ok(Money::new(0, Currency::Usd));
        }

        // Try to get SSD storage pricing
        let filter = PriceFilter {
            provider: Some(*provider),
            resource_type: Some(ResourceType::Storage),
            regions: Some(region.to_string()),
            ..Default::default()
        };

        let prices = self.service.list(filter).await.unwrap_or_default();

        // Look for SSD/premium storage
        if let Some(price) = prices.iter().find(|p| {
            p.service_name.to_lowercase().contains("ssd")
                || p.instance_type
                    .as_ref()
                    .map(|t| t.to_lowercase().contains("ssd") || t.to_lowercase().contains("premium"))
                    .unwrap_or(false)
        }) {
            return Ok(Money::new(
                price.unit_price.amount * total_storage_gb as i64,
                Currency::Usd,
            ));
        }

        // Fallback: estimate at $0.10/GB/month for SSD
        Ok(Money::new(total_storage_gb as i64 * 10, Currency::Usd))
    }

    /// Get managed service cost for comparison
    async fn get_managed_service_cost(
        &self,
        tool: &CncfTool,
        deployment_mode: &DeploymentMode,
        provider: &CloudProvider,
        region: &str,
    ) -> PricingResult<(String, String, Money)> {
        let equivalent = tool
            .managed_equivalents
            .iter()
            .find(|e| e.provider == *provider);

        let (service_name, sku) = match equivalent {
            Some(e) => {
                let sku = match deployment_mode {
                    DeploymentMode::Minimal => &e.minimal_equivalent_sku,
                    DeploymentMode::HighAvailability => &e.ha_equivalent_sku,
                    DeploymentMode::Production => &e.production_equivalent_sku,
                };
                (e.service_name.clone(), sku.clone())
            }
            None => ("Unknown".to_string(), "Unknown".to_string()),
        };

        // Try to find the price in our database
        let filter = PriceFilter {
            provider: Some(*provider),
            resource_type: Some(tool.replaces_resource_type),
            regions: Some(region.to_string()),
            ..Default::default()
        };

        let prices = self.service.list(filter).await.unwrap_or_default();

        // Find matching SKU
        if let Some(price) = prices.iter().find(|p| {
            p.instance_type
                .as_ref()
                .map(|t| t.to_lowercase().contains(&sku.to_lowercase()))
                .unwrap_or(false)
                || p.sku.to_lowercase().contains(&sku.to_lowercase())
        }) {
            // Convert to monthly cost
            let monthly = match price.pricing_unit {
                crate::PricingUnit::Hour => price.unit_price.amount * 730,
                crate::PricingUnit::Month => price.unit_price.amount,
                crate::PricingUnit::GbMonth => price.unit_price.amount * 100, // Assume 100GB
                _ => price.unit_price.amount * 730,
            };
            return Ok((service_name, sku, Money::new(monthly, Currency::Usd)));
        }

        // Fallback estimates based on provider and tool type
        let estimated_monthly = self.estimate_managed_cost(tool, deployment_mode, provider);
        Ok((service_name, sku, Money::new(estimated_monthly, Currency::Usd)))
    }

    /// Estimate managed service cost when not in database
    fn estimate_managed_cost(
        &self,
        tool: &CncfTool,
        deployment_mode: &DeploymentMode,
        provider: &CloudProvider,
    ) -> i64 {
        use crate::cncf_models::CncfToolCategory;

        let base_cost = match (&tool.category, deployment_mode) {
            (CncfToolCategory::Database, DeploymentMode::Minimal) => 2500,      // $25/mo
            (CncfToolCategory::Database, DeploymentMode::HighAvailability) => 15000, // $150/mo
            (CncfToolCategory::Database, DeploymentMode::Production) => 40000,  // $400/mo
            (CncfToolCategory::Cache, DeploymentMode::Minimal) => 1500,         // $15/mo
            (CncfToolCategory::Cache, DeploymentMode::HighAvailability) => 10000, // $100/mo
            (CncfToolCategory::Cache, DeploymentMode::Production) => 30000,     // $300/mo
            (CncfToolCategory::MessageQueue, DeploymentMode::Minimal) => 5000,  // $50/mo
            (CncfToolCategory::MessageQueue, DeploymentMode::HighAvailability) => 30000, // $300/mo
            (CncfToolCategory::MessageQueue, DeploymentMode::Production) => 80000, // $800/mo
            (CncfToolCategory::Storage, DeploymentMode::Minimal) => 1000,       // $10/mo
            (CncfToolCategory::Storage, DeploymentMode::HighAvailability) => 5000, // $50/mo
            (CncfToolCategory::Storage, DeploymentMode::Production) => 20000,   // $200/mo
            _ => 5000, // Default $50/mo
        };

        // Adjust by provider (GCP tends to be cheapest, AWS middle, Azure varies)
        match provider {
            CloudProvider::Gcp => base_cost * 90 / 100,   // 10% cheaper
            CloudProvider::Aws => base_cost,
            CloudProvider::Azure => base_cost * 95 / 100, // 5% cheaper
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_requirements_calculations() {
        let reqs = ResourceRequirements {
            cpu_millicores: 1000,
            memory_mb: 4096,
            storage_gb: 100,
            replicas: 3,
        };

        assert_eq!(reqs.total_cpu_millicores(), 3000);
        assert_eq!(reqs.total_memory_mb(), 12288);
        assert_eq!(reqs.total_storage_gb(), 300);
        assert_eq!(reqs.cpu_cores(), 1.0);
        assert_eq!(reqs.memory_gb(), 4.0);
    }

    #[test]
    fn test_get_cncf_tools() {
        let tools = get_cncf_tools();
        assert!(!tools.is_empty());

        // Check CNPG is present
        let cnpg = tools.iter().find(|t| t.id == "cnpg");
        assert!(cnpg.is_some());

        let cnpg = cnpg.unwrap();
        assert_eq!(cnpg.name, "CloudNativePG");
        assert!(!cnpg.managed_equivalents.is_empty());
    }
}
