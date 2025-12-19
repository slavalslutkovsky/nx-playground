//! CNCF Tool Pricing Models
//!
//! Models for comparing managed cloud services with self-managed
//! CNCF tools running on Kubernetes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{CloudProvider, Money, ResourceType};

/// Category of CNCF tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS, Hash)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum CncfToolCategory {
    /// Database operators (CNPG, Percona, Zalando)
    Database,
    /// Cache/Redis operators
    Cache,
    /// Message queue operators (Strimzi, RabbitMQ)
    MessageQueue,
    /// Storage solutions (Rook-Ceph, Longhorn)
    Storage,
    /// Observability (Prometheus, Grafana)
    Observability,
    /// Service Mesh (Istio, Linkerd)
    ServiceMesh,
    /// GitOps (ArgoCD, Flux)
    GitOps,
}

/// Maturity level of a CNCF project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS, Hash)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum CncfMaturity {
    /// Sandbox projects - early stage
    Sandbox,
    /// Incubating projects - growing adoption
    Incubating,
    /// Graduated projects - production ready
    Graduated,
}

/// Deployment mode for self-managed tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS, Hash)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    /// Single instance, no HA
    Minimal,
    /// High availability setup
    HighAvailability,
    /// Production-grade with all features
    Production,
}

/// Resource requirements for running a CNCF tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ResourceRequirements {
    /// CPU in millicores (1000m = 1 vCPU)
    pub cpu_millicores: i32,
    /// Memory in MB
    pub memory_mb: i32,
    /// Storage in GB (for stateful workloads)
    pub storage_gb: i32,
    /// Number of replicas
    pub replicas: i32,
}

impl ResourceRequirements {
    pub fn cpu_cores(&self) -> f64 {
        self.cpu_millicores as f64 / 1000.0
    }

    pub fn memory_gb(&self) -> f64 {
        self.memory_mb as f64 / 1024.0
    }

    /// Total CPU across all replicas
    pub fn total_cpu_millicores(&self) -> i32 {
        self.cpu_millicores * self.replicas
    }

    /// Total memory across all replicas
    pub fn total_memory_mb(&self) -> i32 {
        self.memory_mb * self.replicas
    }

    /// Total storage across all replicas
    pub fn total_storage_gb(&self) -> i32 {
        self.storage_gb * self.replicas
    }
}

/// CNCF Tool definition with resource requirements
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CncfTool {
    /// Unique identifier (e.g., "cnpg", "strimzi")
    pub id: String,
    /// Display name
    pub name: String,
    /// Tool category
    pub category: CncfToolCategory,
    /// CNCF maturity level
    pub maturity: CncfMaturity,
    /// What cloud resource type this replaces
    pub replaces_resource_type: ResourceType,
    /// GitHub stars (popularity indicator)
    pub github_stars: Option<i32>,
    /// Project URL
    pub project_url: String,
    /// Brief description
    pub description: String,

    // Resource requirements by deployment mode
    /// Operator/controller requirements (runs always)
    pub operator_requirements: ResourceRequirements,
    /// Minimal deployment (dev/test)
    pub minimal_requirements: ResourceRequirements,
    /// HA deployment (staging)
    pub ha_requirements: ResourceRequirements,
    /// Production deployment (full features)
    pub production_requirements: ResourceRequirements,

    /// Estimated monthly ops hours by deployment mode
    pub ops_hours: OpsHoursEstimate,

    /// Equivalent managed services for comparison
    pub managed_equivalents: Vec<ManagedServiceEquivalent>,

    /// Features included (that managed services charge for)
    pub included_features: Vec<String>,
}

/// Estimated operational hours per month
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct OpsHoursEstimate {
    /// Hours for initial setup (one-time)
    pub initial_setup_hours: f32,
    /// Monthly hours for minimal deployment
    pub minimal_monthly_hours: f32,
    /// Monthly hours for HA deployment
    pub ha_monthly_hours: f32,
    /// Monthly hours for production deployment
    pub production_monthly_hours: f32,
}

/// Maps a CNCF tool to equivalent managed services
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ManagedServiceEquivalent {
    pub provider: CloudProvider,
    /// Service name (e.g., "Amazon RDS PostgreSQL")
    pub service_name: String,
    /// Equivalent SKU pattern for minimal deployment
    pub minimal_equivalent_sku: String,
    /// Equivalent SKU pattern for HA deployment
    pub ha_equivalent_sku: String,
    /// Equivalent SKU pattern for production deployment
    pub production_equivalent_sku: String,
}

/// Input for TCO calculation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct TcoCalculationRequest {
    /// CNCF tool to evaluate
    pub tool_id: String,
    /// Deployment mode
    pub deployment_mode: DeploymentMode,
    /// Cloud provider for pricing
    pub provider: CloudProvider,
    /// Region
    pub region: String,
    /// Engineer hourly rate for ops cost calculation
    pub engineer_hourly_rate: Money,
    /// Include control plane cost (set false if shared cluster)
    pub include_control_plane: bool,
    /// Number of similar workloads (amortize ops cost)
    pub workload_count: i32,
}

/// Result of TCO calculation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct TcoCalculationResult {
    pub tool_id: String,
    pub tool_name: String,
    pub deployment_mode: DeploymentMode,
    pub provider: CloudProvider,
    pub region: String,

    // Self-managed costs (monthly)
    /// K8s control plane cost
    pub control_plane_cost: Money,
    /// Compute cost for operator
    pub operator_compute_cost: Money,
    /// Compute cost for workload
    pub workload_compute_cost: Money,
    /// Storage cost (PVCs)
    pub storage_cost: Money,
    /// Estimated backup storage cost
    pub backup_storage_cost: Money,
    /// Total infrastructure cost
    pub total_infra_cost: Money,

    // Operational costs (monthly)
    /// Estimated ops hours per month
    pub ops_hours_per_month: f32,
    /// Ops cost based on engineer rate
    pub ops_cost: Money,
    /// Amortized ops cost (divided by workload count)
    pub amortized_ops_cost: Money,

    // Total self-managed
    pub total_self_managed_cost: Money,

    // Managed service comparison
    pub managed_service_name: String,
    pub managed_service_sku: String,
    pub managed_service_cost: Money,

    // Analysis
    /// Savings (positive) or extra cost (negative) vs managed
    pub savings_vs_managed: Money,
    /// Percentage difference
    pub percentage_difference: f64,
    /// Break-even ops hours (when managed becomes cheaper)
    pub break_even_ops_hours: f32,
    /// Recommendation
    pub recommendation: CostRecommendation,
}

/// Cost comparison recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum CostRecommendation {
    /// Self-managed is significantly cheaper (>30% savings)
    StronglySelfManaged,
    /// Self-managed is somewhat cheaper (10-30% savings)
    ConsiderSelfManaged,
    /// Costs are similar (<10% difference)
    Similar,
    /// Managed service is somewhat cheaper (10-30% savings)
    ConsiderManaged,
    /// Managed service is significantly cheaper (>30% savings)
    StronglyManaged,
}

/// Multi-tool cost comparison
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct InfrastructureCostComparison {
    /// Total monthly cost for all-managed approach
    pub all_managed_cost: Money,
    /// Total monthly cost for all-self-managed approach
    pub all_self_managed_cost: Money,
    /// Hybrid recommendation (best of both)
    pub hybrid_cost: Money,
    /// Per-tool breakdown
    pub tool_comparisons: Vec<TcoCalculationResult>,
    /// Recommended approach for each tool
    pub recommendations: HashMap<String, CostRecommendation>,
}

/// Pre-defined CNCF tool configurations
pub fn get_cncf_tools() -> Vec<CncfTool> {
    vec![
        // CloudNativePG - PostgreSQL Operator
        CncfTool {
            id: "cnpg".to_string(),
            name: "CloudNativePG".to_string(),
            category: CncfToolCategory::Database,
            maturity: CncfMaturity::Sandbox,  // CNCF Sandbox
            replaces_resource_type: ResourceType::Database,
            github_stars: Some(4500),
            project_url: "https://cloudnative-pg.io".to_string(),
            description: "Kubernetes operator for PostgreSQL with HA, backups, and monitoring".to_string(),
            operator_requirements: ResourceRequirements {
                cpu_millicores: 100,
                memory_mb: 256,
                storage_gb: 0,
                replicas: 1,
            },
            minimal_requirements: ResourceRequirements {
                cpu_millicores: 500,
                memory_mb: 1024,
                storage_gb: 20,
                replicas: 1,
            },
            ha_requirements: ResourceRequirements {
                cpu_millicores: 1000,
                memory_mb: 4096,
                storage_gb: 100,
                replicas: 3,
            },
            production_requirements: ResourceRequirements {
                cpu_millicores: 2000,
                memory_mb: 8192,
                storage_gb: 500,
                replicas: 3,
            },
            ops_hours: OpsHoursEstimate {
                initial_setup_hours: 16.0,
                minimal_monthly_hours: 2.0,
                ha_monthly_hours: 4.0,
                production_monthly_hours: 8.0,
            },
            managed_equivalents: vec![
                ManagedServiceEquivalent {
                    provider: CloudProvider::Aws,
                    service_name: "Amazon RDS PostgreSQL".to_string(),
                    minimal_equivalent_sku: "db.t3.micro".to_string(),
                    ha_equivalent_sku: "db.r5.large".to_string(),
                    production_equivalent_sku: "db.r5.xlarge".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Azure,
                    service_name: "Azure Database for PostgreSQL".to_string(),
                    minimal_equivalent_sku: "B1ms".to_string(),
                    ha_equivalent_sku: "D2s_v3".to_string(),
                    production_equivalent_sku: "D4s_v3".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Gcp,
                    service_name: "Cloud SQL PostgreSQL".to_string(),
                    minimal_equivalent_sku: "db-f1-micro".to_string(),
                    ha_equivalent_sku: "db-n1-standard-2".to_string(),
                    production_equivalent_sku: "db-n1-standard-4".to_string(),
                },
            ],
            included_features: vec![
                "High Availability (streaming replication)".to_string(),
                "Automated failover".to_string(),
                "Backup to S3/GCS/Azure Blob".to_string(),
                "Point-in-time recovery".to_string(),
                "Connection pooling (PgBouncer)".to_string(),
                "Prometheus metrics".to_string(),
                "Rolling updates".to_string(),
            ],
        },
        // Dragonfly / Redis Operator
        CncfTool {
            id: "redis-operator".to_string(),
            name: "Redis Operator (Spotahome)".to_string(),
            category: CncfToolCategory::Cache,
            maturity: CncfMaturity::Sandbox,
            replaces_resource_type: ResourceType::Database,
            github_stars: Some(1500),
            project_url: "https://github.com/spotahome/redis-operator".to_string(),
            description: "Kubernetes operator for Redis with sentinel HA".to_string(),
            operator_requirements: ResourceRequirements {
                cpu_millicores: 100,
                memory_mb: 128,
                storage_gb: 0,
                replicas: 1,
            },
            minimal_requirements: ResourceRequirements {
                cpu_millicores: 250,
                memory_mb: 512,
                storage_gb: 0,  // Redis is in-memory
                replicas: 1,
            },
            ha_requirements: ResourceRequirements {
                cpu_millicores: 500,
                memory_mb: 2048,
                storage_gb: 0,
                replicas: 3,  // 1 master + 2 replicas
            },
            production_requirements: ResourceRequirements {
                cpu_millicores: 1000,
                memory_mb: 8192,
                storage_gb: 0,
                replicas: 6,  // 3 masters + 3 replicas (cluster mode)
            },
            ops_hours: OpsHoursEstimate {
                initial_setup_hours: 8.0,
                minimal_monthly_hours: 1.0,
                ha_monthly_hours: 2.0,
                production_monthly_hours: 4.0,
            },
            managed_equivalents: vec![
                ManagedServiceEquivalent {
                    provider: CloudProvider::Aws,
                    service_name: "Amazon ElastiCache Redis".to_string(),
                    minimal_equivalent_sku: "cache.t3.micro".to_string(),
                    ha_equivalent_sku: "cache.r5.large".to_string(),
                    production_equivalent_sku: "cache.r5.xlarge".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Azure,
                    service_name: "Azure Cache for Redis".to_string(),
                    minimal_equivalent_sku: "C0-Basic".to_string(),
                    ha_equivalent_sku: "C1-Standard".to_string(),
                    production_equivalent_sku: "P1-Premium".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Gcp,
                    service_name: "Memorystore Redis".to_string(),
                    minimal_equivalent_sku: "M1".to_string(),
                    ha_equivalent_sku: "M2-Standard".to_string(),
                    production_equivalent_sku: "M5-Standard".to_string(),
                },
            ],
            included_features: vec![
                "Sentinel-based HA".to_string(),
                "Automatic failover".to_string(),
                "Redis Cluster support".to_string(),
                "Prometheus metrics".to_string(),
                "Persistent storage (optional)".to_string(),
            ],
        },
        // Strimzi - Kafka Operator
        CncfTool {
            id: "strimzi".to_string(),
            name: "Strimzi".to_string(),
            category: CncfToolCategory::MessageQueue,
            maturity: CncfMaturity::Incubating,  // CNCF Incubating
            replaces_resource_type: ResourceType::Other,  // No direct equivalent in our enum
            github_stars: Some(4800),
            project_url: "https://strimzi.io".to_string(),
            description: "Kubernetes operator for Apache Kafka".to_string(),
            operator_requirements: ResourceRequirements {
                cpu_millicores: 200,
                memory_mb: 384,
                storage_gb: 0,
                replicas: 1,
            },
            minimal_requirements: ResourceRequirements {
                cpu_millicores: 1000,
                memory_mb: 2048,
                storage_gb: 50,
                replicas: 1,
            },
            ha_requirements: ResourceRequirements {
                cpu_millicores: 2000,
                memory_mb: 4096,
                storage_gb: 200,
                replicas: 3,
            },
            production_requirements: ResourceRequirements {
                cpu_millicores: 4000,
                memory_mb: 8192,
                storage_gb: 1000,
                replicas: 5,
            },
            ops_hours: OpsHoursEstimate {
                initial_setup_hours: 24.0,
                minimal_monthly_hours: 4.0,
                ha_monthly_hours: 8.0,
                production_monthly_hours: 16.0,
            },
            managed_equivalents: vec![
                ManagedServiceEquivalent {
                    provider: CloudProvider::Aws,
                    service_name: "Amazon MSK".to_string(),
                    minimal_equivalent_sku: "kafka.t3.small".to_string(),
                    ha_equivalent_sku: "kafka.m5.large".to_string(),
                    production_equivalent_sku: "kafka.m5.2xlarge".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Azure,
                    service_name: "Azure Event Hubs (Kafka)".to_string(),
                    minimal_equivalent_sku: "Basic".to_string(),
                    ha_equivalent_sku: "Standard".to_string(),
                    production_equivalent_sku: "Premium".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Gcp,
                    service_name: "Confluent Cloud on GCP".to_string(),
                    minimal_equivalent_sku: "Basic".to_string(),
                    ha_equivalent_sku: "Standard".to_string(),
                    production_equivalent_sku: "Dedicated".to_string(),
                },
            ],
            included_features: vec![
                "Kafka cluster management".to_string(),
                "ZooKeeper or KRaft mode".to_string(),
                "Kafka Connect".to_string(),
                "Schema Registry".to_string(),
                "Cruise Control (rebalancing)".to_string(),
                "TLS encryption".to_string(),
                "SASL authentication".to_string(),
                "Prometheus metrics".to_string(),
            ],
        },
        // Rook-Ceph - Storage
        CncfTool {
            id: "rook-ceph".to_string(),
            name: "Rook-Ceph".to_string(),
            category: CncfToolCategory::Storage,
            maturity: CncfMaturity::Graduated,  // CNCF Graduated
            replaces_resource_type: ResourceType::Storage,
            github_stars: Some(12000),
            project_url: "https://rook.io".to_string(),
            description: "Cloud-native storage orchestration for Kubernetes using Ceph".to_string(),
            operator_requirements: ResourceRequirements {
                cpu_millicores: 500,
                memory_mb: 512,
                storage_gb: 0,
                replicas: 1,
            },
            minimal_requirements: ResourceRequirements {
                cpu_millicores: 1000,
                memory_mb: 4096,
                storage_gb: 100,
                replicas: 1,  // Single OSD
            },
            ha_requirements: ResourceRequirements {
                cpu_millicores: 2000,
                memory_mb: 8192,
                storage_gb: 500,
                replicas: 3,  // 3 OSDs
            },
            production_requirements: ResourceRequirements {
                cpu_millicores: 4000,
                memory_mb: 16384,
                storage_gb: 2000,
                replicas: 5,  // 5+ OSDs
            },
            ops_hours: OpsHoursEstimate {
                initial_setup_hours: 40.0,
                minimal_monthly_hours: 4.0,
                ha_monthly_hours: 8.0,
                production_monthly_hours: 16.0,
            },
            managed_equivalents: vec![
                ManagedServiceEquivalent {
                    provider: CloudProvider::Aws,
                    service_name: "Amazon EBS/EFS".to_string(),
                    minimal_equivalent_sku: "gp3".to_string(),
                    ha_equivalent_sku: "io2".to_string(),
                    production_equivalent_sku: "io2-block-express".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Azure,
                    service_name: "Azure Managed Disks".to_string(),
                    minimal_equivalent_sku: "Standard_SSD".to_string(),
                    ha_equivalent_sku: "Premium_SSD".to_string(),
                    production_equivalent_sku: "Ultra_Disk".to_string(),
                },
                ManagedServiceEquivalent {
                    provider: CloudProvider::Gcp,
                    service_name: "Google Persistent Disk".to_string(),
                    minimal_equivalent_sku: "pd-balanced".to_string(),
                    ha_equivalent_sku: "pd-ssd".to_string(),
                    production_equivalent_sku: "pd-extreme".to_string(),
                },
            ],
            included_features: vec![
                "Block storage (RBD)".to_string(),
                "Shared filesystem (CephFS)".to_string(),
                "Object storage (RGW - S3 compatible)".to_string(),
                "Data replication".to_string(),
                "Erasure coding".to_string(),
                "Snapshots and clones".to_string(),
                "Prometheus metrics".to_string(),
            ],
        },
    ]
}
