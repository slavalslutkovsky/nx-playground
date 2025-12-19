//! AI Recommendation Service for CNCF Tools
//!
//! Provides intelligent recommendations for CNCF tools based on
//! various factors including maturity, popularity, and use case fit.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::cncf_client::{
    CategoryRecommendationResponse, CncfToolEnriched, ToolRecommendation,
};
use crate::cncf_models::{CncfMaturity, CncfToolCategory};

/// Trait for AI recommendation providers
#[async_trait]
pub trait AiRecommender: Send + Sync {
    /// Generate recommendations for tools in a category
    async fn recommend_for_category(
        &self,
        category: CncfToolCategory,
        tools: &[CncfToolEnriched],
        use_case: Option<&str>,
    ) -> Result<CategoryRecommendationResponse, RecommenderError>;

    /// Generate pros/cons for a single tool
    async fn analyze_tool(
        &self,
        tool: &CncfToolEnriched,
    ) -> Result<ToolRecommendation, RecommenderError>;
}

/// Error type for recommender
#[derive(Debug, thiserror::Error)]
pub enum RecommenderError {
    #[error("AI service error: {0}")]
    ServiceError(String),
    #[error("No tools to analyze")]
    NoTools,
}

/// Heuristic-based recommender (no external AI service required)
/// Uses maturity, GitHub stars, and known best practices
pub struct HeuristicRecommender {
    /// Pre-defined tool analyses
    known_analyses: HashMap<String, KnownToolAnalysis>,
}

struct KnownToolAnalysis {
    pros: Vec<&'static str>,
    cons: Vec<&'static str>,
    best_for: Vec<&'static str>,
    avoid_if: Vec<&'static str>,
}

impl Default for HeuristicRecommender {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRecommender {
    pub fn new() -> Self {
        let mut known_analyses = HashMap::new();

        // Database tools
        known_analyses.insert(
            "cloudnative-pg".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "Native Kubernetes integration with CRDs",
                    "Automated failover and HA with streaming replication",
                    "Built-in backup to S3/GCS/Azure Blob",
                    "Point-in-time recovery support",
                    "Connection pooling with PgBouncer",
                    "Excellent Prometheus metrics",
                    "Active development and community",
                ],
                cons: vec![
                    "CNCF Sandbox (not yet Incubating)",
                    "Requires Kubernetes expertise",
                    "Manual performance tuning needed",
                    "Less mature than managed services",
                ],
                best_for: vec![
                    "Teams with Kubernetes experience",
                    "Cost-sensitive production workloads",
                    "Multi-cloud or hybrid deployments",
                    "When data sovereignty is required",
                ],
                avoid_if: vec![
                    "Limited DevOps resources",
                    "Need enterprise support SLAs",
                    "Regulatory compliance requires managed service",
                ],
            },
        );

        known_analyses.insert(
            "vitess".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated - production proven",
                    "Horizontal scaling for MySQL",
                    "Used by YouTube, Slack, GitHub",
                    "Supports sharding natively",
                    "Connection pooling built-in",
                ],
                cons: vec![
                    "Complex architecture",
                    "Steep learning curve",
                    "Overkill for small deployments",
                    "MySQL only (no PostgreSQL)",
                ],
                best_for: vec![
                    "Large-scale MySQL deployments",
                    "When sharding is required",
                    "High-traffic applications",
                ],
                avoid_if: vec![
                    "Small to medium workloads",
                    "PostgreSQL is preferred",
                    "Simple HA is sufficient",
                ],
            },
        );

        // Observability tools
        known_analyses.insert(
            "prometheus".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated - industry standard",
                    "Pull-based metrics collection",
                    "Powerful PromQL query language",
                    "Excellent Kubernetes integration",
                    "Huge ecosystem of exporters",
                    "AlertManager for alerting",
                ],
                cons: vec![
                    "Limited long-term storage",
                    "Single-node by default",
                    "High memory usage for large cardinality",
                    "No built-in dashboards",
                ],
                best_for: vec![
                    "Kubernetes-native monitoring",
                    "When Grafana is used for visualization",
                    "Teams familiar with PromQL",
                ],
                avoid_if: vec![
                    "Need long-term metrics retention",
                    "Prefer push-based metrics",
                    "Need built-in visualization",
                ],
            },
        );

        known_analyses.insert(
            "jaeger".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated - production ready",
                    "Distributed tracing standard",
                    "OpenTelemetry compatible",
                    "Scalable architecture",
                    "Good UI for trace analysis",
                ],
                cons: vec![
                    "Storage backend complexity",
                    "High resource usage at scale",
                    "Requires instrumentation effort",
                ],
                best_for: vec![
                    "Microservices debugging",
                    "Performance optimization",
                    "Distributed system observability",
                ],
                avoid_if: vec![
                    "Monolithic applications",
                    "Limited engineering resources",
                ],
            },
        );

        // Message Queue tools
        known_analyses.insert(
            "strimzi".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Incubating - active development",
                    "Full Kafka ecosystem support",
                    "Kafka Connect integration",
                    "Schema Registry support",
                    "Cruise Control for rebalancing",
                    "TLS and SASL authentication",
                ],
                cons: vec![
                    "Complex operations",
                    "High resource requirements",
                    "Kafka expertise needed",
                    "ZooKeeper dependency (unless KRaft)",
                ],
                best_for: vec![
                    "Event-driven architectures",
                    "High-throughput streaming",
                    "When Kafka ecosystem is needed",
                ],
                avoid_if: vec![
                    "Simple pub/sub needs",
                    "Low message volume",
                    "Limited Kafka experience",
                ],
            },
        );

        known_analyses.insert(
            "nats".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Incubating",
                    "Extremely lightweight",
                    "Simple to operate",
                    "JetStream for persistence",
                    "Multi-tenancy support",
                    "Low latency",
                ],
                cons: vec![
                    "Smaller ecosystem than Kafka",
                    "Less tooling available",
                    "JetStream is newer",
                ],
                best_for: vec![
                    "Lightweight messaging",
                    "IoT and edge computing",
                    "Simple pub/sub patterns",
                    "When simplicity is priority",
                ],
                avoid_if: vec![
                    "Complex stream processing needed",
                    "Need Kafka Connect ecosystem",
                ],
            },
        );

        // Storage tools
        known_analyses.insert(
            "rook".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated - production proven",
                    "Orchestrates Ceph on Kubernetes",
                    "Block, file, and object storage",
                    "Self-healing and auto-scaling",
                    "S3-compatible object storage",
                ],
                cons: vec![
                    "Complex to troubleshoot",
                    "High resource requirements",
                    "Ceph expertise helpful",
                    "Significant initial setup",
                ],
                best_for: vec![
                    "Unified storage platform",
                    "When cloud storage is too expensive",
                    "Data sovereignty requirements",
                ],
                avoid_if: vec![
                    "Small clusters",
                    "Limited storage expertise",
                    "Simple PV needs only",
                ],
            },
        );

        known_analyses.insert(
            "longhorn".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Incubating",
                    "Simple to deploy and manage",
                    "Built-in backup to S3",
                    "DR and snapshots",
                    "UI for management",
                    "Lightweight compared to Ceph",
                ],
                cons: vec![
                    "Block storage only",
                    "Performance overhead",
                    "Less features than Ceph",
                ],
                best_for: vec![
                    "Small to medium clusters",
                    "Teams new to cloud-native storage",
                    "When simplicity is priority",
                ],
                avoid_if: vec![
                    "Need object storage (S3)",
                    "Very high performance required",
                    "Large-scale deployments",
                ],
            },
        );

        // Service Mesh tools
        known_analyses.insert(
            "istio".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "Feature-rich service mesh",
                    "Strong security (mTLS)",
                    "Advanced traffic management",
                    "Extensive observability",
                    "Large community",
                ],
                cons: vec![
                    "Resource intensive",
                    "Complex configuration",
                    "Steep learning curve",
                    "Can impact latency",
                ],
                best_for: vec![
                    "Enterprise microservices",
                    "Strong security requirements",
                    "Complex traffic routing",
                ],
                avoid_if: vec![
                    "Small deployments",
                    "Performance-critical low-latency",
                    "Limited DevOps bandwidth",
                ],
            },
        );

        known_analyses.insert(
            "linkerd".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated",
                    "Lightweight and fast",
                    "Simple to operate",
                    "Low resource overhead",
                    "Automatic mTLS",
                    "Rust-based proxy (efficient)",
                ],
                cons: vec![
                    "Fewer features than Istio",
                    "Smaller ecosystem",
                    "Less traffic management options",
                ],
                best_for: vec![
                    "Teams prioritizing simplicity",
                    "Performance-sensitive workloads",
                    "Getting started with service mesh",
                ],
                avoid_if: vec![
                    "Need advanced traffic management",
                    "Complex multi-cluster setups",
                ],
            },
        );

        // GitOps tools
        known_analyses.insert(
            "argo-cd".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated",
                    "Excellent UI",
                    "Multi-cluster support",
                    "SSO integration",
                    "Sync waves and hooks",
                    "Large community",
                ],
                cons: vec![
                    "Can be resource intensive",
                    "Complex RBAC setup",
                    "Application-centric model",
                ],
                best_for: vec![
                    "Teams wanting GitOps with UI",
                    "Multi-cluster deployments",
                    "When visualization matters",
                ],
                avoid_if: vec![
                    "Very resource-constrained",
                    "Prefer CLI-only workflows",
                ],
            },
        );

        known_analyses.insert(
            "flux".to_string(),
            KnownToolAnalysis {
                pros: vec![
                    "CNCF Graduated",
                    "Lightweight",
                    "Native Kubernetes approach",
                    "Multi-tenancy support",
                    "Helm controller built-in",
                    "OCI artifact support",
                ],
                cons: vec![
                    "No built-in UI",
                    "Steeper learning curve",
                    "Less visual feedback",
                ],
                best_for: vec![
                    "CLI-first teams",
                    "Multi-tenant platforms",
                    "Helm-heavy workflows",
                ],
                avoid_if: vec![
                    "Need visual GitOps UI",
                    "Team unfamiliar with Kubernetes",
                ],
            },
        );

        Self { known_analyses }
    }

    fn score_tool(&self, tool: &CncfToolEnriched) -> i32 {
        let mut score = 50; // Base score

        // Maturity bonus
        match tool.maturity {
            CncfMaturity::Graduated => score += 30,
            CncfMaturity::Incubating => score += 15,
            CncfMaturity::Sandbox => score += 0,
        }

        // GitHub stars bonus (if available)
        if let Some(stats) = &tool.github_stats {
            if stats.stars > 10000 {
                score += 15;
            } else if stats.stars > 5000 {
                score += 10;
            } else if stats.stars > 1000 {
                score += 5;
            }
        }

        score.min(100)
    }

    fn generate_generic_pros(&self, tool: &CncfToolEnriched) -> Vec<String> {
        let mut pros = vec![
            "Open source with no licensing costs".to_string(),
            "Runs on any Kubernetes cluster".to_string(),
            "Full control over configuration".to_string(),
        ];

        match tool.maturity {
            CncfMaturity::Graduated => {
                pros.push("CNCF Graduated - production proven".to_string());
            }
            CncfMaturity::Incubating => {
                pros.push("CNCF Incubating - active development".to_string());
            }
            CncfMaturity::Sandbox => {
                pros.push("CNCF Sandbox - early adopter opportunity".to_string());
            }
        }

        if let Some(stats) = &tool.github_stats {
            if stats.stars > 5000 {
                pros.push(format!("Strong community ({} GitHub stars)", stats.stars));
            }
        }

        pros
    }

    fn generate_generic_cons(&self, tool: &CncfToolEnriched) -> Vec<String> {
        let mut cons = vec![
            "Requires Kubernetes expertise".to_string(),
            "Self-managed operational overhead".to_string(),
        ];

        if tool.maturity == CncfMaturity::Sandbox {
            cons.push("Early stage - API may change".to_string());
            cons.push("Smaller community than mature projects".to_string());
        }

        cons
    }
}

#[async_trait]
impl AiRecommender for HeuristicRecommender {
    async fn recommend_for_category(
        &self,
        category: CncfToolCategory,
        tools: &[CncfToolEnriched],
        _use_case: Option<&str>,
    ) -> Result<CategoryRecommendationResponse, RecommenderError> {
        if tools.is_empty() {
            return Err(RecommenderError::NoTools);
        }

        let mut recommendations: Vec<ToolRecommendation> = Vec::new();

        for tool in tools {
            let rec = self.analyze_tool(tool).await?;
            recommendations.push(rec);
        }

        // Sort by score descending
        recommendations.sort_by(|a, b| b.score.cmp(&a.score));

        let top_pick_id = recommendations.first().unwrap().tool_id.clone();
        let top_pick_name = recommendations.first().unwrap().tool_name.clone();
        let top_pick_reason = format!(
            "{} is recommended based on maturity level, community adoption, and feature set. \
             It offers the best balance of stability and capabilities for most use cases.",
            top_pick_name
        );

        Ok(CategoryRecommendationResponse {
            category,
            recommendations,
            top_pick: top_pick_id,
            top_pick_reason,
        })
    }

    async fn analyze_tool(
        &self,
        tool: &CncfToolEnriched,
    ) -> Result<ToolRecommendation, RecommenderError> {
        let score = self.score_tool(tool);

        // Check for known analysis first
        if let Some(known) = self.known_analyses.get(&tool.id) {
            return Ok(ToolRecommendation {
                tool_id: tool.id.clone(),
                tool_name: tool.name.clone(),
                pros: known.pros.iter().map(|s| s.to_string()).collect(),
                cons: known.cons.iter().map(|s| s.to_string()).collect(),
                best_for: known.best_for.iter().map(|s| s.to_string()).collect(),
                avoid_if: known.avoid_if.iter().map(|s| s.to_string()).collect(),
                score,
            });
        }

        // Generate generic recommendations
        Ok(ToolRecommendation {
            tool_id: tool.id.clone(),
            tool_name: tool.name.clone(),
            pros: self.generate_generic_pros(tool),
            cons: self.generate_generic_cons(tool),
            best_for: vec![
                "Teams with Kubernetes experience".to_string(),
                "Cost-conscious deployments".to_string(),
            ],
            avoid_if: vec![
                "Limited DevOps resources".to_string(),
                "Need managed service SLAs".to_string(),
            ],
            score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heuristic_scorer() {
        let recommender = HeuristicRecommender::new();

        let graduated_tool = CncfToolEnriched {
            id: "test".to_string(),
            name: "Test Tool".to_string(),
            category: CncfToolCategory::Database,
            subcategory: None,
            maturity: CncfMaturity::Graduated,
            project_url: String::new(),
            repo_url: None,
            description: String::new(),
            logo_url: None,
            github_stats: Some(crate::cncf_client::GitHubStats {
                stars: 15000,
                ..Default::default()
            }),
            pros: vec![],
            cons: vec![],
            recommendation_score: None,
            updated_at: String::new(),
        };

        let sandbox_tool = CncfToolEnriched {
            maturity: CncfMaturity::Sandbox,
            github_stats: Some(crate::cncf_client::GitHubStats {
                stars: 500,
                ..Default::default()
            }),
            ..graduated_tool.clone()
        };

        assert!(recommender.score_tool(&graduated_tool) > recommender.score_tool(&sandbox_tool));
    }
}
