//! CNCF Landscape Client
//!
//! Fetches real data from the CNCF Landscape and enriches with GitHub stats.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::cncf_models::{CncfMaturity, CncfToolCategory};

/// Raw item from CNCF Landscape JSON
#[derive(Debug, Clone, Deserialize)]
pub struct LandscapeItem {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage_url: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub project: Option<String>, // "graduated", "incubating", "sandbox"
    #[serde(default)]
    pub oss: Option<bool>,
    #[serde(default)]
    pub logo: Option<String>,
}

/// GitHub repository statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS, Default)]
#[ts(export)]
pub struct GitHubStats {
    pub stars: i32,
    pub forks: i32,
    pub open_issues: i32,
    pub watchers: i32,
    pub last_commit: Option<String>,
    pub contributors_count: Option<i32>,
    pub license: Option<String>,
    pub language: Option<String>,
}

/// Enriched CNCF tool with real data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CncfToolEnriched {
    /// Unique identifier derived from name
    pub id: String,
    /// Display name
    pub name: String,
    /// Category from landscape
    pub category: CncfToolCategory,
    /// Subcategory for more detail
    pub subcategory: Option<String>,
    /// CNCF maturity level
    pub maturity: CncfMaturity,
    /// Project URL
    pub project_url: String,
    /// GitHub repository URL
    pub repo_url: Option<String>,
    /// Brief description
    pub description: String,
    /// Logo URL
    pub logo_url: Option<String>,
    /// GitHub statistics
    pub github_stats: Option<GitHubStats>,
    /// AI-generated pros
    pub pros: Vec<String>,
    /// AI-generated cons
    pub cons: Vec<String>,
    /// AI recommendation score (0-100)
    pub recommendation_score: Option<i32>,
    /// When data was last updated
    pub updated_at: String,
}

/// A category of CNCF tools with recommendations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CncfCategoryGroup {
    /// Category identifier
    pub category: CncfToolCategory,
    /// Display name
    pub display_name: String,
    /// Category description
    pub description: String,
    /// Tools in this category, sorted by recommendation
    pub tools: Vec<CncfToolEnriched>,
    /// AI-recommended best tool for this category
    pub recommended_tool_id: Option<String>,
    /// Explanation of recommendation
    pub recommendation_reason: Option<String>,
}

/// Response for listing all CNCF tools grouped by category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CncfToolsResponse {
    /// Tools grouped by category
    pub categories: Vec<CncfCategoryGroup>,
    /// Total number of tools
    pub total_tools: usize,
    /// When data was last fetched
    pub last_updated: String,
}

/// CNCF Landscape client for fetching real data
pub struct CncfLandscapeClient {
    client: Client,
    github_token: Option<String>,
}

impl CncfLandscapeClient {
    const LANDSCAPE_URL: &'static str = "https://landscape.cncf.io/data/items.json";
    const GITHUB_API: &'static str = "https://api.github.com";

    pub fn new(github_token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            github_token,
        }
    }

    /// Fetch all CNCF projects from the landscape
    pub async fn fetch_landscape(&self) -> Result<Vec<LandscapeItem>, CncfClientError> {
        info!("Fetching CNCF Landscape data");

        let response = self
            .client
            .get(Self::LANDSCAPE_URL)
            .header("User-Agent", "cloud-cost-optimizer")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CncfClientError::ApiError(format!(
                "Landscape API returned {}",
                response.status()
            )));
        }

        let items: Vec<LandscapeItem> = response.json().await?;
        info!("Fetched {} items from CNCF Landscape", items.len());
        Ok(items)
    }

    /// Fetch GitHub stats for a repository
    pub async fn fetch_github_stats(&self, repo_url: &str) -> Result<GitHubStats, CncfClientError> {
        // Extract owner/repo from URL
        let repo_path = extract_github_repo(repo_url)
            .ok_or_else(|| CncfClientError::InvalidRepo(repo_url.to_string()))?;

        let url = format!("{}/repos/{}", Self::GITHUB_API, repo_path);
        debug!("Fetching GitHub stats from: {}", url);

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "cloud-cost-optimizer")
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(token) = &self.github_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            warn!("GitHub API returned {} for {}", response.status(), repo_path);
            return Ok(GitHubStats::default());
        }

        let data: serde_json::Value = response.json().await?;

        Ok(GitHubStats {
            stars: data["stargazers_count"].as_i64().unwrap_or(0) as i32,
            forks: data["forks_count"].as_i64().unwrap_or(0) as i32,
            open_issues: data["open_issues_count"].as_i64().unwrap_or(0) as i32,
            watchers: data["subscribers_count"].as_i64().unwrap_or(0) as i32,
            last_commit: data["pushed_at"].as_str().map(|s| s.to_string()),
            contributors_count: None, // Requires separate API call
            license: data["license"]["spdx_id"].as_str().map(|s| s.to_string()),
            language: data["language"].as_str().map(|s| s.to_string()),
        })
    }

    /// Fetch tools for specific categories relevant to cost optimization
    pub async fn fetch_cost_relevant_tools(&self) -> Result<CncfToolsResponse, CncfClientError> {
        let items = self.fetch_landscape().await?;

        // Filter to CNCF projects only (graduated, incubating, sandbox)
        let cncf_projects: Vec<_> = items
            .into_iter()
            .filter(|item| item.project.is_some())
            .collect();

        info!("Found {} CNCF projects", cncf_projects.len());

        // Map landscape categories to our categories
        let mut category_map: HashMap<CncfToolCategory, Vec<CncfToolEnriched>> = HashMap::new();

        for item in cncf_projects {
            if let Some(our_category) = map_landscape_category(&item.category, item.subcategory.as_deref()) {
                let tool = CncfToolEnriched {
                    id: slugify(&item.name),
                    name: item.name.clone(),
                    category: our_category,
                    subcategory: item.subcategory.clone(),
                    maturity: parse_maturity(item.project.as_deref()),
                    project_url: item.homepage_url.clone().unwrap_or_default(),
                    repo_url: item.repo_url.clone(),
                    description: item.description.clone().unwrap_or_else(|| format!("{} - CNCF Project", item.name)),
                    logo_url: item.logo.map(|l| format!("https://landscape.cncf.io/logos/{}", l)),
                    github_stats: None,
                    pros: Vec::new(),
                    cons: Vec::new(),
                    recommendation_score: None,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                };

                category_map
                    .entry(our_category)
                    .or_default()
                    .push(tool);
            }
        }

        // Build category groups
        let categories: Vec<CncfCategoryGroup> = vec![
            CncfToolCategory::Database,
            CncfToolCategory::Cache,
            CncfToolCategory::MessageQueue,
            CncfToolCategory::Storage,
            CncfToolCategory::Observability,
            CncfToolCategory::ServiceMesh,
            CncfToolCategory::GitOps,
        ]
        .into_iter()
        .filter_map(|cat| {
            let tools = category_map.remove(&cat).unwrap_or_default();
            if tools.is_empty() {
                return None;
            }
            Some(CncfCategoryGroup {
                category: cat,
                display_name: category_display_name(cat),
                description: category_description(cat),
                tools,
                recommended_tool_id: None,
                recommendation_reason: None,
            })
        })
        .collect();

        let total_tools = categories.iter().map(|c| c.tools.len()).sum();

        Ok(CncfToolsResponse {
            categories,
            total_tools,
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Enrich tools with GitHub stats (rate-limited)
    pub async fn enrich_with_github(&self, tools: &mut [CncfToolEnriched]) {
        for tool in tools.iter_mut() {
            if let Some(repo_url) = &tool.repo_url {
                match self.fetch_github_stats(repo_url).await {
                    Ok(stats) => {
                        tool.github_stats = Some(stats);
                    }
                    Err(e) => {
                        warn!("Failed to fetch GitHub stats for {}: {}", tool.name, e);
                    }
                }
                // Rate limiting - GitHub allows 60 req/hour unauthenticated
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}

/// Map CNCF Landscape categories to our categories
fn map_landscape_category(category: &str, subcategory: Option<&str>) -> Option<CncfToolCategory> {
    let cat_lower = category.to_lowercase();
    let subcat_lower = subcategory.map(|s| s.to_lowercase());

    // Database
    if cat_lower.contains("database") || subcat_lower.as_deref().map_or(false, |s| s.contains("database")) {
        return Some(CncfToolCategory::Database);
    }

    // Storage
    if cat_lower.contains("storage") || subcat_lower.as_deref().map_or(false, |s| s.contains("storage")) {
        return Some(CncfToolCategory::Storage);
    }

    // Message Queue / Streaming
    if cat_lower.contains("streaming")
        || cat_lower.contains("messaging")
        || subcat_lower.as_deref().map_or(false, |s| s.contains("streaming") || s.contains("messaging"))
    {
        return Some(CncfToolCategory::MessageQueue);
    }

    // Observability
    if cat_lower.contains("observability")
        || cat_lower.contains("monitoring")
        || subcat_lower.as_deref().map_or(false, |s| s.contains("observability") || s.contains("monitoring") || s.contains("logging") || s.contains("tracing"))
    {
        return Some(CncfToolCategory::Observability);
    }

    // Service Mesh
    if subcat_lower.as_deref().map_or(false, |s| s.contains("service mesh")) {
        return Some(CncfToolCategory::ServiceMesh);
    }

    // GitOps / CD
    if subcat_lower.as_deref().map_or(false, |s| s.contains("continuous") || s.contains("gitops")) {
        return Some(CncfToolCategory::GitOps);
    }

    None
}

fn parse_maturity(project: Option<&str>) -> CncfMaturity {
    match project {
        Some("graduated") => CncfMaturity::Graduated,
        Some("incubating") => CncfMaturity::Incubating,
        Some("sandbox") => CncfMaturity::Sandbox,
        _ => CncfMaturity::Sandbox,
    }
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}

fn extract_github_repo(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/');
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() > 1 {
            let repo_path = parts[1].trim_start_matches('/');
            // Take only owner/repo
            let components: Vec<&str> = repo_path.split('/').collect();
            if components.len() >= 2 {
                return Some(format!("{}/{}", components[0], components[1]));
            }
        }
    }
    None
}

fn category_display_name(cat: CncfToolCategory) -> String {
    match cat {
        CncfToolCategory::Database => "Databases".to_string(),
        CncfToolCategory::Cache => "Caching & In-Memory".to_string(),
        CncfToolCategory::MessageQueue => "Message Queues & Streaming".to_string(),
        CncfToolCategory::Storage => "Cloud Native Storage".to_string(),
        CncfToolCategory::Observability => "Observability & Monitoring".to_string(),
        CncfToolCategory::ServiceMesh => "Service Mesh".to_string(),
        CncfToolCategory::GitOps => "GitOps & Continuous Delivery".to_string(),
    }
}

fn category_description(cat: CncfToolCategory) -> String {
    match cat {
        CncfToolCategory::Database => "Self-managed database operators that can replace RDS, Cloud SQL, Azure Database".to_string(),
        CncfToolCategory::Cache => "In-memory caching solutions replacing ElastiCache, Memorystore, Azure Cache".to_string(),
        CncfToolCategory::MessageQueue => "Message queuing and event streaming replacing MSK, Pub/Sub, Event Hubs".to_string(),
        CncfToolCategory::Storage => "Cloud-native storage solutions replacing EBS, Azure Disk, Persistent Disk".to_string(),
        CncfToolCategory::Observability => "Monitoring, logging, and tracing replacing CloudWatch, Azure Monitor, Cloud Monitoring".to_string(),
        CncfToolCategory::ServiceMesh => "Service mesh solutions for microservices networking and security".to_string(),
        CncfToolCategory::GitOps => "GitOps tools for declarative, version-controlled deployments".to_string(),
    }
}

/// Errors from CNCF client
#[derive(Debug, thiserror::Error)]
pub enum CncfClientError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid repository URL: {0}")]
    InvalidRepo(String),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// AI recommendation request for a category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryRecommendationRequest {
    pub category: CncfToolCategory,
    pub tools: Vec<ToolSummary>,
    pub use_case: Option<String>,
}

/// Summary of a tool for AI recommendation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolSummary {
    pub name: String,
    pub description: String,
    pub maturity: CncfMaturity,
    pub github_stars: Option<i32>,
    pub license: Option<String>,
}

/// AI-generated recommendation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ToolRecommendation {
    pub tool_id: String,
    pub tool_name: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub best_for: Vec<String>,
    pub avoid_if: Vec<String>,
    pub score: i32, // 0-100
}

/// AI recommendation response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CategoryRecommendationResponse {
    pub category: CncfToolCategory,
    pub recommendations: Vec<ToolRecommendation>,
    pub top_pick: String,
    pub top_pick_reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_github_repo() {
        assert_eq!(
            extract_github_repo("https://github.com/cloudnative-pg/cloudnative-pg"),
            Some("cloudnative-pg/cloudnative-pg".to_string())
        );
        assert_eq!(
            extract_github_repo("https://github.com/strimzi/strimzi-kafka-operator/"),
            Some("strimzi/strimzi-kafka-operator".to_string())
        );
        assert_eq!(extract_github_repo("https://gitlab.com/foo/bar"), None);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("CloudNativePG"), "cloudnativepg");
        assert_eq!(slugify("Rook Ceph"), "rook-ceph");
        assert_eq!(slugify("Argo CD"), "argo-cd");
    }

    #[test]
    fn test_map_category() {
        assert_eq!(
            map_landscape_category("App Definition and Development", Some("Database")),
            Some(CncfToolCategory::Database)
        );
        assert_eq!(
            map_landscape_category("Runtime", Some("Cloud Native Storage")),
            Some(CncfToolCategory::Storage)
        );
        assert_eq!(
            map_landscape_category("Observability and Analysis", Some("Monitoring")),
            Some(CncfToolCategory::Observability)
        );
    }
}
