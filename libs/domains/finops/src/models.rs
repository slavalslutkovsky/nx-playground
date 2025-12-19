use chrono::{DateTime, Utc};
use domain_pricing::{CloudProvider, Money};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use strum::{Display, EnumString};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Chat session status
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default, ToSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SessionStatus {
    #[default]
    Active,
    Archived,
    Completed,
}

/// Message role in conversation
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default, ToSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MessageRole {
    #[default]
    User,
    Assistant,
    System,
    Tool,
}

/// Cloud account connection status
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default, ToSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CloudAccountStatus {
    #[default]
    Pending,
    Connected,
    Error,
    Disconnected,
}

/// Recommendation type
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default, ToSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RecommendationType {
    #[default]
    Rightsize,
    Terminate,
    Migrate,
    Reserve,
    Upgrade,
    Consolidate,
}

/// Recommendation status
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default, ToSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RecommendationStatus {
    #[default]
    Pending,
    Approved,
    Applied,
    Dismissed,
    Failed,
}

/// Chat context for session preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, TS)]
#[ts(export)]
pub struct ChatContext {
    /// Preferred cloud providers
    #[serde(default)]
    pub preferred_providers: Vec<CloudProvider>,
    /// Monthly budget constraint (in cents)
    pub budget_monthly: Option<i64>,
    /// Preferred regions
    #[serde(default)]
    pub regions: Vec<String>,
    /// Compliance requirements (e.g., "hipaa", "pci-dss")
    #[serde(default)]
    pub compliance_requirements: Vec<String>,
    /// Connected cloud account IDs for this session
    #[serde(default)]
    #[ts(as = "Vec<String>")]
    pub cloud_account_ids: Vec<Uuid>,
}

/// Chat session for conversation persistence
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ChatSession {
    #[ts(as = "String")]
    pub id: Uuid,
    #[ts(as = "Option<String>")]
    pub user_id: Option<Uuid>,
    pub title: Option<String>,
    pub context: ChatContext,
    pub status: SessionStatus,
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
    #[ts(as = "String")]
    pub updated_at: DateTime<Utc>,
}

/// Tool call record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ToolCallRecord {
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub latency_ms: Option<i64>,
}

/// Chat message with tool calls
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ChatMessage {
    #[ts(as = "String")]
    pub id: Uuid,
    #[ts(as = "String")]
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallRecord>>,
    pub token_count: Option<i32>,
    pub latency_ms: Option<i32>,
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
}

/// Connected cloud account for resource exploration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CloudAccount {
    #[ts(as = "String")]
    pub id: Uuid,
    #[ts(as = "String")]
    pub user_id: Uuid,
    pub provider: CloudProvider,
    pub account_id: String,
    pub name: Option<String>,
    pub regions: Vec<String>,
    #[ts(as = "Option<String>")]
    pub last_sync_at: Option<DateTime<Utc>>,
    pub status: CloudAccountStatus,
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
}

/// Resource specifications
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, TS)]
#[ts(export)]
pub struct ResourceSpecs {
    pub instance_type: Option<String>,
    pub vcpus: Option<i32>,
    pub memory_gb: Option<f64>,
    pub storage_gb: Option<f64>,
    pub state: Option<String>,
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: Option<serde_json::Map<String, JsonValue>>,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Utilization {
    pub cpu_avg: Option<f64>,
    pub memory_avg: Option<f64>,
    pub network_in_bytes: Option<i64>,
    pub network_out_bytes: Option<i64>,
    pub storage_used_bytes: Option<i64>,
}

/// Cached cloud resource inventory
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct CloudResource {
    #[ts(as = "String")]
    pub id: Uuid,
    #[ts(as = "String")]
    pub account_id: Uuid,
    pub resource_id: String,
    pub resource_type: String,
    pub region: String,
    pub name: Option<String>,
    pub specs: ResourceSpecs,
    pub monthly_cost_cents: Option<i64>,
    pub utilization: Option<Utilization>,
    #[ts(skip)]
    pub tags: Option<serde_json::Map<String, JsonValue>>,
    #[ts(as = "String")]
    pub last_seen_at: DateTime<Utc>,
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Recommendation {
    #[ts(as = "String")]
    pub id: Uuid,
    #[ts(as = "Option<String>")]
    pub session_id: Option<Uuid>,
    #[ts(as = "Option<String>")]
    pub resource_id: Option<Uuid>,
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub current_cost_cents: Option<i64>,
    pub projected_cost_cents: Option<i64>,
    pub savings_cents: Option<i64>,
    pub confidence: Option<f32>,
    pub details: Option<RecommendationDetails>,
    pub status: RecommendationStatus,
    #[ts(as = "String")]
    pub created_at: DateTime<Utc>,
}

/// Recommendation details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct RecommendationDetails {
    pub risks: Vec<String>,
    pub implementation_steps: Vec<String>,
    pub estimated_downtime: Option<String>,
    pub target_provider: Option<CloudProvider>,
    pub target_region: Option<String>,
    pub target_instance_type: Option<String>,
}

// ===== Request/Response DTOs =====

/// Chat request from frontend
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChatRequest {
    pub session_id: Option<Uuid>,
    /// User ID is optional - null for guest/unauthenticated users
    pub user_id: Option<Uuid>,
    #[validate(length(min = 1, max = 10000))]
    pub message: String,
    pub context: Option<ChatContext>,
}

/// Chat response chunk for streaming
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChatChunk {
    Text { content: String },
    ToolCall { name: String, arguments: String },
    ToolResult { name: String, result: String },
    Done { session_id: Uuid },
    Error { message: String },
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ChatResponse {
    #[ts(as = "String")]
    pub session_id: Uuid,
    pub content: String,
    pub tool_calls: Vec<ToolCallRecord>,
}

/// Create session request
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateSession {
    pub user_id: Option<Uuid>,
    pub title: Option<String>,
    pub context: Option<ChatContext>,
}

/// Create cloud account request
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateCloudAccount {
    pub user_id: Uuid,
    pub provider: CloudProvider,
    #[validate(length(min = 1, max = 255))]
    pub account_id: String,
    pub name: Option<String>,
    pub regions: Vec<String>,
    /// Base64 encoded encrypted credentials
    pub credentials: Option<String>,
}

/// Session filter
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, Default)]
pub struct SessionFilter {
    pub user_id: Option<Uuid>,
    pub status: Option<SessionStatus>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

/// Resource filter
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, Default)]
pub struct ResourceFilter {
    pub account_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub region: Option<String>,
    pub min_cost: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

/// Recommendation filter
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams, Default)]
pub struct RecommendationFilter {
    pub session_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub recommendation_type: Option<RecommendationType>,
    pub status: Option<RecommendationStatus>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

/// Resource summary for aggregation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ResourceSummary {
    pub total_count: i64,
    pub total_monthly_cost: Money,
    pub by_type: std::collections::HashMap<String, TypeSummary>,
    pub by_region: std::collections::HashMap<String, RegionSummary>,
}

/// Type summary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct TypeSummary {
    pub count: i64,
    pub monthly_cost: Money,
}

/// Region summary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct RegionSummary {
    pub count: i64,
    pub monthly_cost: Money,
}

/// Optimization summary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct OptimizationSummary {
    pub total_recommendations: i64,
    pub total_monthly_savings: Money,
    pub total_annual_savings: Money,
    pub by_type: std::collections::HashMap<String, i64>,
    pub top_recommendations: Vec<Recommendation>,
}

fn default_limit() -> usize {
    50
}
