//! FinOps agent tools
//!
//! Tools that the AI agent can use to gather information and perform actions.

use async_trait::async_trait;
use domain_pricing::{models::PriceComparison, CloudProvider, PricingService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{FinopsError, FinopsResult};
use crate::models::{CloudResource, Recommendation, RecommendationDetails, RecommendationType};
use crate::repository::FinopsRepository;
use crate::service::FinopsService;

/// Trait for FinOps tools that can be executed by the agent
#[async_trait]
pub trait FinopsTool: Send + Sync {
    /// Tool name for identification
    fn name(&self) -> &str;

    /// Description for the LLM to understand when to use this tool
    fn description(&self) -> &str;

    /// JSON schema for the tool's input parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given arguments
    async fn execute(&self, arguments: serde_json::Value) -> FinopsResult<String>;
}

// =============================================================================
// Compare Prices Tool
// =============================================================================

/// Input for the compare_prices tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ComparePricesInput {
    /// Resource type to compare (compute, database, storage, kubernetes)
    pub resource_type: String,
    /// Minimum vCPUs required
    pub vcpus: Option<i32>,
    /// Minimum memory in GB
    pub memory_gb: Option<i32>,
    /// Regions to compare (e.g., ["us-east-1", "westus2"])
    pub regions: Option<Vec<String>>,
    /// Providers to include (aws, azure, gcp)
    pub providers: Option<Vec<String>>,
}

/// Tool to compare prices across cloud providers
pub struct ComparePricesTool<R: domain_pricing::repository::PricingRepository> {
    service: Arc<PricingService<R>>,
}

impl<R: domain_pricing::repository::PricingRepository + 'static> ComparePricesTool<R> {
    pub fn new(service: Arc<PricingService<R>>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<R: domain_pricing::repository::PricingRepository + 'static> FinopsTool
    for ComparePricesTool<R>
{
    fn name(&self) -> &str {
        "compare_prices"
    }

    fn description(&self) -> &str {
        "Compare cloud service prices across providers (AWS, Azure, GCP) for a specific resource type"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "resource_type": {
                    "type": "string",
                    "description": "Resource type to compare: compute, database, storage, kubernetes, serverless",
                    "enum": ["compute", "database", "storage", "kubernetes", "serverless"]
                },
                "vcpus": {
                    "type": "integer",
                    "description": "Minimum vCPUs required"
                },
                "memory_gb": {
                    "type": "integer",
                    "description": "Minimum memory in GB"
                },
                "regions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Regions to compare (e.g., us-east-1, westus2)"
                },
                "providers": {
                    "type": "array",
                    "items": { "type": "string", "enum": ["aws", "azure", "gcp"] },
                    "description": "Providers to include in comparison"
                }
            },
            "required": ["resource_type"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> FinopsResult<String> {
        let input: ComparePricesInput = serde_json::from_value(arguments)
            .map_err(|e| FinopsError::InvalidInput(e.to_string()))?;

        let resource_type: domain_pricing::ResourceType = input
            .resource_type
            .parse()
            .map_err(|_| FinopsError::InvalidInput("Invalid resource type".to_string()))?;

        let providers: Vec<CloudProvider> = input
            .providers
            .unwrap_or_default()
            .into_iter()
            .filter_map(|p| p.parse().ok())
            .collect();

        let comparisons = self
            .service
            .compare_prices(
                resource_type,
                input.vcpus,
                input.memory_gb,
                input.regions.unwrap_or_default(),
                providers,
            )
            .await
            .map_err(FinopsError::Pricing)?;

        // Format result as markdown
        let result = format_price_comparison(&comparisons);
        Ok(result)
    }
}

fn format_price_comparison(comparisons: &[PriceComparison]) -> String {
    let mut output = String::new();

    for comparison in comparisons {
        output.push_str(&format!("## Price Comparison: {}\n\n", comparison.comparison_key));

        output.push_str("| Provider | Instance Type | Monthly Cost | Region |\n");
        output.push_str("|----------|---------------|--------------|--------|\n");

        for pp in &comparison.provider_prices {
            output.push_str(&format!(
                "| {} | {} | ${:.2} | {} |\n",
                pp.provider,
                pp.price.instance_type.as_deref().unwrap_or("-"),
                pp.monthly_estimate.to_decimal(),
                pp.price.region
            ));
        }

        if let Some(cheapest) = &comparison.cheapest {
            output.push_str(&format!(
                "\n**Cheapest Option**: {} - ${:.2}/month\n",
                cheapest.provider,
                cheapest.monthly_estimate.to_decimal()
            ));
        }

        if let Some(savings) = &comparison.potential_savings {
            output.push_str(&format!(
                "**Potential Savings**: ${:.2}/month (${:.2}/year)\n",
                savings.to_decimal(),
                savings.to_decimal() * 12.0
            ));
        }

        output.push('\n');
    }

    output
}

// =============================================================================
// Explore Resources Tool
// =============================================================================

/// Input for the explore_resources tool
#[derive(Debug, Deserialize, Serialize)]
pub struct ExploreResourcesInput {
    /// Cloud account ID to explore
    pub account_id: Option<Uuid>,
    /// Resource type filter
    pub resource_type: Option<String>,
    /// Region filter
    pub region: Option<String>,
    /// Minimum monthly cost to include
    pub min_cost: Option<i64>,
}

/// Tool to explore client's cloud resources
pub struct ExploreResourcesTool<R: FinopsRepository> {
    service: Arc<FinopsService<R>>,
}

impl<R: FinopsRepository + 'static> ExploreResourcesTool<R> {
    pub fn new(service: Arc<FinopsService<R>>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<R: FinopsRepository + 'static> FinopsTool for ExploreResourcesTool<R> {
    fn name(&self) -> &str {
        "explore_resources"
    }

    fn description(&self) -> &str {
        "Explore and analyze client's cloud resources from connected accounts"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "account_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "Cloud account ID to explore"
                },
                "resource_type": {
                    "type": "string",
                    "description": "Filter by resource type (ec2_instance, rds_instance, etc.)"
                },
                "region": {
                    "type": "string",
                    "description": "Filter by region"
                },
                "min_cost": {
                    "type": "integer",
                    "description": "Minimum monthly cost in cents"
                }
            }
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> FinopsResult<String> {
        let input: ExploreResourcesInput = serde_json::from_value(arguments)
            .map_err(|e| FinopsError::InvalidInput(e.to_string()))?;

        let filter = crate::models::ResourceFilter {
            account_id: input.account_id,
            resource_type: input.resource_type,
            region: input.region,
            min_cost: input.min_cost,
            limit: 50,
            offset: 0,
        };

        let resources = self.service.list_resources(filter).await?;

        // Format result as markdown
        let result = format_resources(&resources);
        Ok(result)
    }
}

fn format_resources(resources: &[CloudResource]) -> String {
    if resources.is_empty() {
        return "No resources found matching the criteria.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("## Cloud Resources ({} found)\n\n", resources.len()));

    // Calculate totals
    let total_cost: i64 = resources.iter().filter_map(|r| r.monthly_cost_cents).sum();

    output.push_str("| Name | Type | Region | Monthly Cost | Utilization |\n");
    output.push_str("|------|------|--------|--------------|-------------|\n");

    for resource in resources.iter().take(20) {
        let name = resource.name.as_deref().unwrap_or(&resource.resource_id);
        let cost = resource
            .monthly_cost_cents
            .map(|c| format!("${:.2}", c as f64 / 100.0))
            .unwrap_or_else(|| "-".to_string());
        let util = resource
            .utilization
            .as_ref()
            .and_then(|u| u.cpu_avg)
            .map(|c| format!("{:.1}% CPU", c))
            .unwrap_or_else(|| "-".to_string());

        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            truncate(name, 30),
            resource.resource_type,
            resource.region,
            cost,
            util
        ));
    }

    if resources.len() > 20 {
        output.push_str(&format!("\n*...and {} more resources*\n", resources.len() - 20));
    }

    output.push_str(&format!(
        "\n**Total Monthly Cost**: ${:.2}\n",
        total_cost as f64 / 100.0
    ));

    output
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// =============================================================================
// Generate Recommendation Tool
// =============================================================================

/// Input for the generate_recommendation tool
#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateRecommendationInput {
    /// Resource ID to analyze
    pub resource_id: Uuid,
    /// Type of recommendation to generate
    pub recommendation_type: String,
    /// Target provider for migration (if applicable)
    pub target_provider: Option<String>,
    /// Target region (if applicable)
    pub target_region: Option<String>,
}

/// Output from recommendation generation
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct GenerateRecommendationOutput {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub current_cost_cents: i64,
    pub projected_cost_cents: i64,
    pub savings_cents: i64,
    pub confidence: f32,
    pub risks: Vec<String>,
    pub implementation_steps: Vec<String>,
}

/// Tool to generate optimization recommendations
pub struct GenerateRecommendationTool<R: FinopsRepository> {
    service: Arc<FinopsService<R>>,
}

impl<R: FinopsRepository + 'static> GenerateRecommendationTool<R> {
    pub fn new(service: Arc<FinopsService<R>>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<R: FinopsRepository + 'static> FinopsTool for GenerateRecommendationTool<R> {
    fn name(&self) -> &str {
        "generate_recommendation"
    }

    fn description(&self) -> &str {
        "Generate an optimization recommendation for a specific resource"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "resource_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "Resource ID to analyze"
                },
                "recommendation_type": {
                    "type": "string",
                    "enum": ["rightsize", "terminate", "migrate", "reserve", "upgrade"],
                    "description": "Type of recommendation to generate"
                },
                "target_provider": {
                    "type": "string",
                    "enum": ["aws", "azure", "gcp"],
                    "description": "Target provider for migration"
                },
                "target_region": {
                    "type": "string",
                    "description": "Target region for migration"
                }
            },
            "required": ["resource_id", "recommendation_type"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> FinopsResult<String> {
        let input: GenerateRecommendationInput = serde_json::from_value(arguments)
            .map_err(|e| FinopsError::InvalidInput(e.to_string()))?;

        // Get the resource
        let resource = self.service.get_resource(input.resource_id).await?;

        // Generate recommendation based on type
        let rec_type: RecommendationType = input
            .recommendation_type
            .parse()
            .unwrap_or(RecommendationType::Rightsize);

        let recommendation = generate_recommendation_for_resource(&resource, rec_type)?;

        // Save the recommendation
        let saved = self.service.create_recommendation(recommendation).await?;

        // Format output
        let output = format_recommendation(&saved);
        Ok(output)
    }
}

fn generate_recommendation_for_resource(
    resource: &CloudResource,
    rec_type: RecommendationType,
) -> FinopsResult<Recommendation> {
    let current_cost = resource.monthly_cost_cents.unwrap_or(0);

    // Estimate savings based on recommendation type
    let (projected_cost, confidence, risks, steps) = match rec_type {
        RecommendationType::Rightsize => {
            let savings_pct = 0.3; // 30% savings estimate
            let projected = (current_cost as f64 * (1.0 - savings_pct)) as i64;
            (
                projected,
                0.85,
                vec![
                    "May impact performance during peak usage".to_string(),
                    "Requires instance restart".to_string(),
                ],
                vec![
                    "Create snapshot/backup of the resource".to_string(),
                    "Schedule maintenance window".to_string(),
                    "Apply size change".to_string(),
                    "Monitor performance for 24 hours".to_string(),
                ],
            )
        }
        RecommendationType::Terminate => (
            0,
            0.95,
            vec!["Data will be permanently deleted".to_string()],
            vec![
                "Verify resource is not in use".to_string(),
                "Create backup if needed".to_string(),
                "Delete the resource".to_string(),
            ],
        ),
        RecommendationType::Reserve => {
            let savings_pct = 0.4; // 40% savings with reserved
            let projected = (current_cost as f64 * (1.0 - savings_pct)) as i64;
            (
                projected,
                0.9,
                vec![
                    "Commitment period required (1 or 3 years)".to_string(),
                    "Less flexibility to change instance types".to_string(),
                ],
                vec![
                    "Review usage patterns for stability".to_string(),
                    "Choose commitment term (1 or 3 years)".to_string(),
                    "Purchase reserved capacity".to_string(),
                ],
            )
        }
        RecommendationType::Migrate => {
            let savings_pct = 0.25;
            let projected = (current_cost as f64 * (1.0 - savings_pct)) as i64;
            (
                projected,
                0.7,
                vec![
                    "Migration requires downtime".to_string(),
                    "Application compatibility needs verification".to_string(),
                    "Data transfer costs apply".to_string(),
                ],
                vec![
                    "Assess application compatibility".to_string(),
                    "Plan migration window".to_string(),
                    "Set up target infrastructure".to_string(),
                    "Migrate data and test".to_string(),
                    "Switch traffic and decommission old resources".to_string(),
                ],
            )
        }
        _ => (current_cost, 0.5, vec![], vec![]),
    };

    let savings = current_cost - projected_cost;

    Ok(Recommendation {
        id: Uuid::now_v7(),
        session_id: None,
        resource_id: Some(resource.id),
        recommendation_type: rec_type,
        title: format!(
            "{} {} for cost savings",
            rec_type,
            resource.name.as_deref().unwrap_or(&resource.resource_id)
        ),
        description: format!(
            "Recommended action for {} ({}) in {}",
            resource.resource_type, resource.resource_id, resource.region
        ),
        current_cost_cents: Some(current_cost),
        projected_cost_cents: Some(projected_cost),
        savings_cents: Some(savings),
        confidence: Some(confidence),
        details: Some(RecommendationDetails {
            risks,
            implementation_steps: steps,
            estimated_downtime: Some("5-30 minutes".to_string()),
            target_provider: None,
            target_region: None,
            target_instance_type: None,
        }),
        status: crate::models::RecommendationStatus::Pending,
        created_at: chrono::Utc::now(),
    })
}

fn format_recommendation(rec: &Recommendation) -> String {
    let mut output = String::new();

    output.push_str(&format!("## Recommendation: {}\n\n", rec.title));
    output.push_str(&format!("{}\n\n", rec.description));

    if let (Some(current), Some(projected), Some(savings)) =
        (rec.current_cost_cents, rec.projected_cost_cents, rec.savings_cents)
    {
        output.push_str("### Cost Analysis\n\n");
        output.push_str(&format!(
            "| Metric | Value |\n|--------|-------|\n| Current Cost | ${:.2}/mo |\n| Projected Cost | ${:.2}/mo |\n| **Monthly Savings** | **${:.2}** |\n| **Annual Savings** | **${:.2}** |\n\n",
            current as f64 / 100.0,
            projected as f64 / 100.0,
            savings as f64 / 100.0,
            savings as f64 * 12.0 / 100.0
        ));
    }

    if let Some(confidence) = rec.confidence {
        output.push_str(&format!("**Confidence**: {:.0}%\n\n", confidence * 100.0));
    }

    if let Some(details) = &rec.details {
        if !details.risks.is_empty() {
            output.push_str("### Risks\n\n");
            for risk in &details.risks {
                output.push_str(&format!("- {}\n", risk));
            }
            output.push('\n');
        }

        if !details.implementation_steps.is_empty() {
            output.push_str("### Implementation Steps\n\n");
            for (i, step) in details.implementation_steps.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, step));
            }
        }
    }

    output
}

// =============================================================================
// Tool Registry
// =============================================================================

/// Collection of all available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn FinopsTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register<T: FinopsTool + 'static>(&mut self, tool: T) {
        self.tools.push(Box::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<&dyn FinopsTool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    pub fn list(&self) -> Vec<(&str, &str, serde_json::Value)> {
        self.tools
            .iter()
            .map(|t| (t.name(), t.description(), t.parameters_schema()))
            .collect()
    }

    pub async fn execute(&self, name: &str, arguments: serde_json::Value) -> FinopsResult<String> {
        let tool = self
            .get(name)
            .ok_or_else(|| FinopsError::ToolExecution(format!("Tool not found: {}", name)))?;

        tool.execute(arguments).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
