# FinOps AI Agent Implementation Plan

## Executive Summary

Build an AI-powered FinOps assistant that helps clients optimize cloud infrastructure costs through intelligent recommendations, cross-provider comparison, resource analysis, and eventually automated migrations.

## Current Architecture Analysis

### What You Have

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CURRENT ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SolidJS (apps/cloud/web)          Rust API (apps/zerg/api)         │
│  ├── Dashboard                      ├── /api/prices (REST)          │
│  ├── TCO Calculator                 ├── /api/tasks (gRPC)           │
│  ├── CNCF Tools                     ├── /api/tasks-stream (Redis)   │
│  ├── Price Comparison               └── PostgreSQL + Redis          │
│  └── Price Finder                                                    │
│                                                                      │
│  Pricing Domain (libs/domains/pricing)                               │
│  ├── PriceEntry, CloudProvider, Money models                        │
│  ├── TCO Calculator (self-managed vs managed)                       │
│  ├── CNCF Landscape integration                                     │
│  ├── HeuristicRecommender (rule-based)                              │
│  └── Price comparison across providers                              │
│                                                                      │
│  Infrastructure                                                      │
│  ├── gRPC services (zerg-tasks)                                     │
│  ├── Redis Streams (async processing)                               │
│  ├── PostgreSQL (SeaORM)                                            │
│  └── Proto definitions (manifests/grpc/)                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Gaps to Fill

| Gap | Solution |
|-----|----------|
| No chat interface | Add chat component with TanStack AI |
| No LLM integration | Add Rig for Rust-native LLM agents |
| No cloud API integration | Add AWS/Azure/GCP SDK clients |
| No conversation persistence | Add chat sessions/messages tables |
| No agent orchestration | Multi-agent gRPC architecture |
| No write operations | Phased approach with approval workflow |

---

## Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FINOPS AGENT ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    SOLIDJS FRONTEND                                  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │    │
│  │  │ Chat Widget  │  │  Dashboard   │  │  Resource Explorer       │  │    │
│  │  │ TanStack AI  │  │  (existing)  │  │  (new)                   │  │    │
│  │  └──────┬───────┘  └──────────────┘  └──────────────────────────┘  │    │
│  └─────────┼───────────────────────────────────────────────────────────┘    │
│            │ SSE/WebSocket                                                   │
│            ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    RUST API (Axum) - Orchestrator                    │    │
│  │  ┌──────────────────────────────────────────────────────────────┐   │    │
│  │  │  /api/finops/chat          - Chat endpoint (SSE streaming)   │   │    │
│  │  │  /api/finops/sessions      - Session management              │   │    │
│  │  │  /api/finops/explore       - Resource exploration            │   │    │
│  │  │  /api/finops/optimize      - Optimization suggestions        │   │    │
│  │  │  /api/finops/migrate       - Migration planning (Phase 2)    │   │    │
│  │  └──────────────────────────────────────────────────────────────┘   │    │
│  │                              │                                       │    │
│  │                    ┌─────────┴─────────┐                            │    │
│  │                    ▼                   ▼                            │    │
│  │            ┌───────────────┐   ┌───────────────┐                    │    │
│  │            │ Orchestrator  │   │ Tool Registry │                    │    │
│  │            │ Agent (Rig)   │   │ (Functions)   │                    │    │
│  │            └───────┬───────┘   └───────────────┘                    │    │
│  └────────────────────┼─────────────────────────────────────────────────┘    │
│                       │ gRPC                                                 │
│            ┌──────────┼──────────┬──────────────────┐                       │
│            ▼          ▼          ▼                  ▼                       │
│  ┌──────────────┐ ┌──────────┐ ┌──────────────┐ ┌──────────────┐           │
│  │   Pricing    │ │ Resource │ │  Optimizer   │ │  Migration   │           │
│  │   Agent      │ │ Explorer │ │    Agent     │ │    Agent     │           │
│  │   (Rust)     │ │  Agent   │ │   (Rust)     │ │  (Phase 2)   │           │
│  └──────┬───────┘ └────┬─────┘ └──────┬───────┘ └──────────────┘           │
│         │              │              │                                     │
│         ▼              ▼              ▼                                     │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                        │
│  │ cloud_prices │ │ Cloud APIs   │ │ Rules Engine │                        │
│  │ PostgreSQL   │ │ AWS/GCP/Azure│ │ + ML Models  │                        │
│  └──────────────┘ └──────────────┘ └──────────────┘                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic chat interface with pricing data access

#### 1.1 Database Schema

```sql
-- libs/migration/src/m20251220_000000_create_finops_chat.rs

-- Chat sessions for conversation persistence
CREATE TABLE finops_chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    title VARCHAR(255),
    context JSONB DEFAULT '{}',  -- Client preferences, selected providers, etc.
    status VARCHAR(50) DEFAULT 'active',  -- active, archived, completed
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Chat messages with tool calls
CREATE TABLE finops_chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES finops_chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,  -- user, assistant, system, tool
    content TEXT,
    tool_calls JSONB,           -- [{name, arguments, result}]
    token_count INTEGER,
    latency_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Connected cloud accounts for resource exploration
CREATE TABLE finops_cloud_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    provider VARCHAR(20) NOT NULL,  -- aws, azure, gcp
    account_id VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    credentials_encrypted BYTEA,    -- Encrypted with app key
    regions TEXT[],                 -- Enabled regions
    last_sync_at TIMESTAMPTZ,
    status VARCHAR(50) DEFAULT 'pending',  -- pending, connected, error
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, provider, account_id)
);

-- Cached resource inventory
CREATE TABLE finops_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID REFERENCES finops_cloud_accounts(id) ON DELETE CASCADE,
    resource_id VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,  -- ec2_instance, rds_instance, etc.
    region VARCHAR(50) NOT NULL,
    name VARCHAR(255),
    specs JSONB NOT NULL,          -- {vcpus, memory_gb, storage_gb, ...}
    monthly_cost_cents BIGINT,
    utilization JSONB,             -- {cpu_avg, memory_avg, ...}
    tags JSONB,
    last_seen_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(account_id, resource_id)
);

-- Optimization recommendations
CREATE TABLE finops_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES finops_chat_sessions(id),
    resource_id UUID REFERENCES finops_resources(id),
    recommendation_type VARCHAR(50),  -- rightsize, terminate, migrate, reserve
    current_cost_cents BIGINT,
    projected_cost_cents BIGINT,
    savings_cents BIGINT,
    confidence REAL,
    details JSONB,
    status VARCHAR(50) DEFAULT 'pending',  -- pending, approved, applied, dismissed
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_chat_sessions_user ON finops_chat_sessions(user_id);
CREATE INDEX idx_chat_messages_session ON finops_chat_messages(session_id);
CREATE INDEX idx_resources_account ON finops_resources(account_id);
CREATE INDEX idx_resources_type ON finops_resources(resource_type);
CREATE INDEX idx_recommendations_session ON finops_recommendations(session_id);
```

#### 1.2 Domain Structure

```
libs/domains/finops/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── models.rs           # ChatSession, ChatMessage, CloudAccount, etc.
│   ├── error.rs            # FinopsError, FinopsResult
│   ├── repository.rs       # Repository trait
│   ├── postgres.rs         # PostgreSQL implementation
│   ├── service.rs          # Business logic
│   ├── handlers.rs         # HTTP handlers with OpenAPI
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── orchestrator.rs # Main agent that routes to specialists
│   │   ├── tools.rs        # Tool definitions for Rig
│   │   └── prompts.rs      # System prompts and templates
│   └── cloud/
│       ├── mod.rs
│       ├── aws.rs          # AWS SDK client
│       ├── azure.rs        # Azure SDK client
│       └── gcp.rs          # GCP SDK client
```

#### 1.3 Core Agent with Rig

```rust
// libs/domains/finops/src/agent/orchestrator.rs
use rig::{
    agent::Agent,
    providers::openai::{Client, GPT_4O},
    tool::Tool,
};
use crate::agent::tools::*;

pub struct FinopsOrchestrator {
    agent: Agent,
    pricing_service: Arc<PricingService>,
    db: DatabasePool,
}

impl FinopsOrchestrator {
    pub async fn new(
        pricing_service: Arc<PricingService>,
        db: DatabasePool,
    ) -> Result<Self> {
        let client = Client::from_env();

        let agent = client
            .agent(GPT_4O)
            .preamble(FINOPS_SYSTEM_PROMPT)
            .tool(ComparePricesTool::new(pricing_service.clone()))
            .tool(SearchPricesTool::new(pricing_service.clone()))
            .tool(CalculateTcoTool::new(pricing_service.clone()))
            .tool(GetCncfAlternativesTool::new(pricing_service.clone()))
            .tool(ExploreResourcesTool::new(db.clone()))
            .tool(AnalyzeUtilizationTool::new(db.clone()))
            .tool(GenerateRecommendationTool::new(db.clone()))
            .build();

        Ok(Self { agent, pricing_service, db })
    }

    pub async fn chat(
        &self,
        session_id: Uuid,
        message: &str,
        context: &SessionContext,
    ) -> Result<ChatResponse> {
        // Build context from session
        let context_prompt = self.build_context_prompt(context);

        // Execute agent
        let response = self.agent
            .prompt(&format!("{}\n\nUser: {}", context_prompt, message))
            .await?;

        Ok(ChatResponse {
            content: response.content,
            tool_calls: response.tool_calls,
        })
    }

    pub fn chat_stream(
        &self,
        session_id: Uuid,
        message: &str,
        context: &SessionContext,
    ) -> impl Stream<Item = Result<ChatChunk>> {
        // Streaming implementation
        async_stream::stream! {
            let mut stream = self.agent.prompt_stream(message).await?;
            while let Some(chunk) = stream.next().await {
                yield Ok(ChatChunk::Text(chunk));
            }
        }
    }
}

const FINOPS_SYSTEM_PROMPT: &str = r#"
You are a FinOps AI assistant specializing in cloud cost optimization. Your role is to:

1. **Analyze Pricing**: Compare cloud service prices across AWS, Azure, and GCP
2. **Recommend Solutions**: Suggest optimal infrastructure configurations
3. **Calculate TCO**: Provide total cost of ownership analysis
4. **Explore Resources**: Analyze client's existing cloud resources
5. **Optimize Costs**: Identify savings opportunities

## Available Tools

- `compare_prices`: Compare prices for a resource type across providers
- `search_prices`: Search for specific service pricing
- `calculate_tco`: Calculate TCO for self-managed vs managed services
- `get_cncf_alternatives`: Find open-source alternatives to managed services
- `explore_resources`: List and analyze client's cloud resources
- `analyze_utilization`: Check resource utilization metrics
- `generate_recommendation`: Create optimization recommendations

## Guidelines

- Always provide specific numbers and percentages
- Compare at least 2-3 options when making recommendations
- Consider both cost AND operational overhead
- Ask clarifying questions when requirements are unclear
- For migrations, outline risks and dependencies
- Be conservative with savings estimates

## Response Format

- Use markdown for formatting
- Include tables for comparisons
- Provide actionable next steps
- Cite data sources (pricing data date, API source)
"#;
```

#### 1.4 Tool Definitions

```rust
// libs/domains/finops/src/agent/tools.rs
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Tool 1: Compare Prices
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ComparePricesInput {
    /// Resource type to compare (compute, database, storage, kubernetes)
    pub resource_type: String,
    /// Minimum vCPUs required
    pub vcpus: Option<i32>,
    /// Minimum memory in GB
    pub memory_gb: Option<i32>,
    /// Regions to compare (e.g., ["us-east-1", "westus2", "us-central1"])
    pub regions: Option<Vec<String>>,
    /// Providers to include (aws, azure, gcp)
    pub providers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ComparePricesOutput {
    pub comparisons: Vec<ProviderComparison>,
    pub cheapest: ProviderPrice,
    pub savings_vs_most_expensive: f64,
    pub recommendation: String,
}

pub struct ComparePricesTool {
    service: Arc<PricingService>,
}

impl Tool for ComparePricesTool {
    const NAME: &'static str = "compare_prices";
    type Input = ComparePricesInput;
    type Output = ComparePricesOutput;
    type Error = FinopsError;

    fn description(&self) -> String {
        "Compare cloud service prices across providers for a specific resource type".into()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let comparison = self.service.compare_prices(
            input.resource_type.parse()?,
            input.vcpus,
            input.memory_gb,
            input.regions.unwrap_or_default(),
            input.providers.unwrap_or_default(),
        ).await?;

        let cheapest = comparison.cheapest();
        let most_expensive = comparison.most_expensive();
        let savings = (most_expensive.monthly_cost - cheapest.monthly_cost)
            / most_expensive.monthly_cost * 100.0;

        Ok(ComparePricesOutput {
            comparisons: comparison.providers,
            cheapest,
            savings_vs_most_expensive: savings,
            recommendation: self.generate_recommendation(&comparison),
        })
    }
}

// Tool 2: Explore Resources
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExploreResourcesInput {
    /// Cloud account ID to explore
    pub account_id: Option<Uuid>,
    /// Resource type filter
    pub resource_type: Option<String>,
    /// Region filter
    pub region: Option<String>,
    /// Minimum monthly cost to include
    pub min_cost: Option<i64>,
    /// Include utilization data
    pub include_utilization: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ExploreResourcesOutput {
    pub resources: Vec<ResourceSummary>,
    pub total_monthly_cost: Money,
    pub by_type: HashMap<String, TypeSummary>,
    pub by_region: HashMap<String, RegionSummary>,
    pub optimization_opportunities: Vec<String>,
}

pub struct ExploreResourcesTool {
    db: DatabasePool,
}

impl Tool for ExploreResourcesTool {
    const NAME: &'static str = "explore_resources";
    type Input = ExploreResourcesInput;
    type Output = ExploreResourcesOutput;
    type Error = FinopsError;

    fn description(&self) -> String {
        "Explore and analyze client's cloud resources across connected accounts".into()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let resources = self.fetch_resources(&input).await?;

        // Aggregate by type and region
        let by_type = self.aggregate_by_type(&resources);
        let by_region = self.aggregate_by_region(&resources);

        // Identify optimization opportunities
        let opportunities = self.identify_opportunities(&resources).await?;

        Ok(ExploreResourcesOutput {
            total_monthly_cost: resources.iter().map(|r| r.monthly_cost).sum(),
            resources,
            by_type,
            by_region,
            optimization_opportunities: opportunities,
        })
    }
}

// Tool 3: Generate Recommendation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateRecommendationInput {
    /// Resource to optimize
    pub resource_id: Uuid,
    /// Type of recommendation (rightsize, migrate, reserve, terminate)
    pub recommendation_type: String,
    /// Target provider for migration (if applicable)
    pub target_provider: Option<String>,
    /// Target region for migration (if applicable)
    pub target_region: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateRecommendationOutput {
    pub recommendation_id: Uuid,
    pub current_state: ResourceState,
    pub recommended_state: ResourceState,
    pub monthly_savings: Money,
    pub annual_savings: Money,
    pub confidence: f64,
    pub risks: Vec<String>,
    pub implementation_steps: Vec<String>,
    pub estimated_downtime: Option<String>,
}

// Tool 4: Calculate Infrastructure Cost
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateInfraCostInput {
    /// Infrastructure requirements
    pub requirements: InfraRequirements,
    /// Providers to consider
    pub providers: Vec<String>,
    /// Regions to consider
    pub regions: Vec<String>,
    /// Include reserved instance pricing
    pub include_reserved: Option<bool>,
    /// Include spot/preemptible pricing
    pub include_spot: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InfraRequirements {
    /// Compute instances needed
    pub compute: Vec<ComputeRequirement>,
    /// Database requirements
    pub databases: Vec<DatabaseRequirement>,
    /// Storage requirements in GB
    pub storage_gb: i64,
    /// Expected data transfer GB/month
    pub data_transfer_gb: Option<i64>,
    /// Kubernetes cluster requirements
    pub kubernetes: Option<KubernetesRequirement>,
}
```

#### 1.5 HTTP Handlers with Streaming

```rust
// libs/domains/finops/src/handlers.rs
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;

pub fn router(state: FinopsState) -> Router {
    Router::new()
        // Chat endpoints
        .route("/chat", post(chat_handler))
        .route("/chat/stream", post(chat_stream_handler))

        // Session management
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/:id", get(get_session).delete(delete_session))
        .route("/sessions/:id/messages", get(get_messages))

        // Cloud accounts
        .route("/accounts", get(list_accounts).post(connect_account))
        .route("/accounts/:id", get(get_account).delete(disconnect_account))
        .route("/accounts/:id/sync", post(sync_account))

        // Resources
        .route("/resources", get(list_resources))
        .route("/resources/:id", get(get_resource))
        .route("/resources/:id/recommendations", get(get_recommendations))

        // Quick actions (no chat)
        .route("/compare", post(quick_compare))
        .route("/optimize", post(quick_optimize))

        .with_state(state)
}

// Streaming chat handler
#[utoipa::path(
    post,
    path = "/chat/stream",
    tag = "finops",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "SSE stream of chat responses"),
    )
)]
async fn chat_stream_handler(
    State(state): State<FinopsState>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        // Load or create session
        let session = state.service
            .get_or_create_session(request.session_id, request.user_id)
            .await
            .unwrap();

        // Save user message
        state.service
            .save_message(&session.id, "user", &request.message, None)
            .await
            .unwrap();

        // Stream agent response
        let mut agent_stream = state.orchestrator
            .chat_stream(session.id, &request.message, &session.context)
            .await
            .unwrap();

        let mut full_response = String::new();
        let mut tool_calls = Vec::new();

        while let Some(chunk) = agent_stream.next().await {
            match chunk {
                Ok(ChatChunk::Text(text)) => {
                    full_response.push_str(&text);
                    yield Ok(Event::default()
                        .event("text")
                        .data(&text));
                }
                Ok(ChatChunk::ToolCall { name, args, result }) => {
                    tool_calls.push(ToolCallRecord { name, args, result });
                    yield Ok(Event::default()
                        .event("tool")
                        .data(serde_json::to_string(&tool_calls.last()).unwrap()));
                }
                Ok(ChatChunk::Done) => {
                    // Save assistant message
                    state.service
                        .save_message(
                            &session.id,
                            "assistant",
                            &full_response,
                            Some(&tool_calls),
                        )
                        .await
                        .unwrap();

                    yield Ok(Event::default()
                        .event("done")
                        .data(""));
                }
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(e.to_string()));
                }
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<Uuid>,
    pub user_id: Uuid,
    pub message: String,
    pub context: Option<ChatContext>,
}

#[derive(Debug, Deserialize)]
pub struct ChatContext {
    pub preferred_providers: Option<Vec<String>>,
    pub budget_monthly: Option<i64>,
    pub regions: Option<Vec<String>>,
    pub compliance_requirements: Option<Vec<String>>,
}
```

---

### Phase 2: Cloud Integration (Weeks 3-4)

**Goal**: Connect to client cloud accounts and explore resources

#### 2.1 Cloud Provider SDKs

```toml
# libs/domains/finops/Cargo.toml
[dependencies]
# AWS
aws-config = "1.5"
aws-sdk-ec2 = "1.89"
aws-sdk-rds = "1.83"
aws-sdk-s3 = "1.65"
aws-sdk-cloudwatch = "1.62"
aws-sdk-costexplorer = "1.58"
aws-sdk-pricing = "1.56"

# Azure
azure_identity = "0.21"
azure_mgmt_compute = "0.21"
azure_mgmt_storage = "0.21"
azure_mgmt_sql = "0.21"
azure_mgmt_costmanagement = "0.21"

# GCP
google-cloud-sdk = "0.4"
```

#### 2.2 Cloud Resource Explorer

```rust
// libs/domains/finops/src/cloud/aws.rs
use aws_sdk_ec2 as ec2;
use aws_sdk_rds as rds;
use aws_sdk_cloudwatch as cloudwatch;

pub struct AwsExplorer {
    ec2_client: ec2::Client,
    rds_client: rds::Client,
    cloudwatch_client: cloudwatch::Client,
    region: String,
}

impl AwsExplorer {
    pub async fn new(credentials: &AwsCredentials, region: &str) -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials.to_provider())
            .region(Region::new(region.to_string()))
            .load()
            .await;

        Ok(Self {
            ec2_client: ec2::Client::new(&config),
            rds_client: rds::Client::new(&config),
            cloudwatch_client: cloudwatch::Client::new(&config),
            region: region.to_string(),
        })
    }

    pub async fn list_ec2_instances(&self) -> Result<Vec<CloudResource>> {
        let response = self.ec2_client
            .describe_instances()
            .send()
            .await?;

        let mut resources = Vec::new();
        for reservation in response.reservations() {
            for instance in reservation.instances() {
                let instance_id = instance.instance_id().unwrap_or_default();
                let instance_type = instance.instance_type()
                    .map(|t| t.as_str())
                    .unwrap_or_default();

                // Get utilization metrics
                let utilization = self.get_instance_utilization(instance_id).await?;

                resources.push(CloudResource {
                    resource_id: instance_id.to_string(),
                    resource_type: "ec2_instance".to_string(),
                    region: self.region.clone(),
                    name: self.get_name_tag(instance.tags()),
                    specs: json!({
                        "instance_type": instance_type,
                        "vcpus": self.get_vcpus(instance_type),
                        "memory_gb": self.get_memory_gb(instance_type),
                        "state": instance.state().map(|s| s.name().map(|n| n.as_str())),
                    }),
                    utilization: Some(utilization),
                    tags: self.tags_to_json(instance.tags()),
                    monthly_cost_cents: self.estimate_monthly_cost(instance_type).await?,
                });
            }
        }

        Ok(resources)
    }

    async fn get_instance_utilization(&self, instance_id: &str) -> Result<Utilization> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(7);

        let cpu_response = self.cloudwatch_client
            .get_metric_statistics()
            .namespace("AWS/EC2")
            .metric_name("CPUUtilization")
            .dimensions(
                Dimension::builder()
                    .name("InstanceId")
                    .value(instance_id)
                    .build(),
            )
            .start_time(start_time.into())
            .end_time(end_time.into())
            .period(3600) // 1 hour
            .statistics(Statistic::Average)
            .send()
            .await?;

        let cpu_avg = cpu_response
            .datapoints()
            .iter()
            .filter_map(|d| d.average())
            .sum::<f64>()
            / cpu_response.datapoints().len().max(1) as f64;

        Ok(Utilization {
            cpu_avg,
            memory_avg: None, // Requires CloudWatch agent
            network_in_bytes: None,
            network_out_bytes: None,
            storage_used_bytes: None,
        })
    }
}
```

#### 2.3 Resource Sync Worker

```rust
// apps/zerg/finops-worker/src/main.rs
use domain_finops::cloud::{AwsExplorer, AzureExplorer, GcpExplorer};
use stream_worker::{Worker, StreamConsumer};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let db = database::postgres::connect(&config.database_url).await?;
    let redis = database::redis::connect(&config.redis_url).await?;

    let worker = Worker::new(
        redis,
        "finops:sync",
        SyncHandler { db },
    );

    worker.run().await
}

struct SyncHandler {
    db: DatabasePool,
}

#[async_trait]
impl StreamConsumer for SyncHandler {
    type Message = SyncRequest;

    async fn process(&self, message: Self::Message) -> Result<()> {
        let account = self.db
            .finops_cloud_accounts()
            .find_by_id(message.account_id)
            .await?;

        let credentials = decrypt_credentials(&account.credentials_encrypted)?;

        let resources = match account.provider.as_str() {
            "aws" => {
                let mut all_resources = Vec::new();
                for region in &account.regions {
                    let explorer = AwsExplorer::new(&credentials, region).await?;
                    all_resources.extend(explorer.list_all_resources().await?);
                }
                all_resources
            }
            "azure" => {
                let explorer = AzureExplorer::new(&credentials).await?;
                explorer.list_all_resources().await?
            }
            "gcp" => {
                let explorer = GcpExplorer::new(&credentials).await?;
                explorer.list_all_resources().await?
            }
            _ => return Err(anyhow!("Unknown provider")),
        };

        // Upsert resources
        self.db
            .finops_resources()
            .upsert_many(&resources)
            .await?;

        // Update sync timestamp
        self.db
            .finops_cloud_accounts()
            .update_last_sync(message.account_id, Utc::now())
            .await?;

        Ok(())
    }
}
```

---

### Phase 3: Optimization Engine (Weeks 5-6)

**Goal**: Intelligent recommendations based on resource analysis

#### 3.1 Optimization Rules Engine

```rust
// libs/domains/finops/src/optimizer/rules.rs

pub trait OptimizationRule: Send + Sync {
    fn name(&self) -> &str;
    fn applies_to(&self, resource: &CloudResource) -> bool;
    fn analyze(&self, resource: &CloudResource, context: &OptContext) -> Option<Recommendation>;
}

// Rule 1: Underutilized Compute
pub struct UnderutilizedComputeRule {
    cpu_threshold: f64,      // Default 20%
    memory_threshold: f64,   // Default 30%
    min_observation_days: i32,
}

impl OptimizationRule for UnderutilizedComputeRule {
    fn name(&self) -> &str {
        "underutilized_compute"
    }

    fn applies_to(&self, resource: &CloudResource) -> bool {
        matches!(resource.resource_type.as_str(), "ec2_instance" | "azure_vm" | "gce_instance")
    }

    fn analyze(&self, resource: &CloudResource, ctx: &OptContext) -> Option<Recommendation> {
        let util = resource.utilization.as_ref()?;

        if util.cpu_avg < self.cpu_threshold {
            let current_type = resource.specs.get("instance_type")?.as_str()?;
            let smaller_type = ctx.find_smaller_instance(current_type)?;

            let current_cost = resource.monthly_cost_cents;
            let new_cost = ctx.get_instance_price(&smaller_type)?;
            let savings = current_cost - new_cost;

            return Some(Recommendation {
                recommendation_type: RecommendationType::Rightsize,
                title: format!("Rightsize {} to {}", current_type, smaller_type),
                description: format!(
                    "Instance {} is using only {:.1}% CPU on average. \
                     Consider downsizing from {} to {}.",
                    resource.name.as_deref().unwrap_or(&resource.resource_id),
                    util.cpu_avg,
                    current_type,
                    smaller_type,
                ),
                current_cost_cents: current_cost,
                projected_cost_cents: new_cost,
                savings_cents: savings,
                confidence: self.calculate_confidence(util),
                risks: vec![
                    "May impact performance during peak usage".to_string(),
                    "Requires instance restart".to_string(),
                ],
                implementation_steps: vec![
                    format!("1. Create AMI/snapshot of {}", resource.resource_id),
                    format!("2. Stop instance {}", resource.resource_id),
                    format!("3. Change instance type to {}", smaller_type),
                    "4. Start instance and verify application".to_string(),
                ],
            });
        }

        None
    }
}

// Rule 2: Unattached Storage
pub struct UnattachedStorageRule;

impl OptimizationRule for UnattachedStorageRule {
    fn name(&self) -> &str {
        "unattached_storage"
    }

    fn applies_to(&self, resource: &CloudResource) -> bool {
        matches!(resource.resource_type.as_str(), "ebs_volume" | "azure_disk" | "gce_disk")
    }

    fn analyze(&self, resource: &CloudResource, _ctx: &OptContext) -> Option<Recommendation> {
        let attached = resource.specs.get("attached")?.as_bool()?;

        if !attached {
            return Some(Recommendation {
                recommendation_type: RecommendationType::Terminate,
                title: format!("Delete unattached volume {}", resource.resource_id),
                description: format!(
                    "Volume {} ({} GB) has been unattached for over 30 days.",
                    resource.resource_id,
                    resource.specs.get("size_gb").unwrap_or(&json!(0)),
                ),
                current_cost_cents: resource.monthly_cost_cents,
                projected_cost_cents: 0,
                savings_cents: resource.monthly_cost_cents,
                confidence: 0.95,
                risks: vec!["Data will be permanently deleted".to_string()],
                implementation_steps: vec![
                    "1. Verify volume is not needed".to_string(),
                    "2. Create snapshot if data might be needed".to_string(),
                    format!("3. Delete volume {}", resource.resource_id),
                ],
            });
        }

        None
    }
}

// Rule 3: Reserved Instance Opportunity
pub struct ReservedInstanceRule {
    min_utilization_hours: i32,  // 700+ hours/month = good RI candidate
    min_months_running: i32,     // 3+ months = stable workload
}

// Rule 4: Cross-Provider Migration
pub struct CrossProviderMigrationRule {
    savings_threshold: f64,  // 20%+ savings to recommend migration
}

impl OptimizationRule for CrossProviderMigrationRule {
    fn analyze(&self, resource: &CloudResource, ctx: &OptContext) -> Option<Recommendation> {
        // Find equivalent in other providers
        let current_provider = &resource.provider;
        let other_providers: Vec<_> = ["aws", "azure", "gcp"]
            .iter()
            .filter(|p| *p != current_provider)
            .collect();

        let mut best_alternative: Option<(String, String, i64)> = None;

        for provider in other_providers {
            if let Some(equivalent) = ctx.find_equivalent(resource, provider) {
                let alt_cost = ctx.get_price(&equivalent)?;
                let savings = resource.monthly_cost_cents - alt_cost;
                let savings_pct = savings as f64 / resource.monthly_cost_cents as f64;

                if savings_pct > self.savings_threshold {
                    if best_alternative.is_none()
                        || savings > best_alternative.as_ref().unwrap().2
                    {
                        best_alternative = Some((
                            provider.to_string(),
                            equivalent,
                            savings,
                        ));
                    }
                }
            }
        }

        best_alternative.map(|(provider, equivalent, savings)| Recommendation {
            recommendation_type: RecommendationType::Migrate,
            title: format!("Migrate to {} for {:.0}% savings", provider,
                (savings as f64 / resource.monthly_cost_cents as f64) * 100.0),
            description: format!(
                "Consider migrating {} to {} ({}) for estimated monthly savings of ${}.",
                resource.resource_id,
                provider,
                equivalent,
                savings / 100,
            ),
            current_cost_cents: resource.monthly_cost_cents,
            projected_cost_cents: resource.monthly_cost_cents - savings,
            savings_cents: savings,
            confidence: 0.7,  // Lower confidence for migrations
            risks: vec![
                "Migration requires downtime".to_string(),
                "Application compatibility needs verification".to_string(),
                "Data transfer costs apply".to_string(),
            ],
            implementation_steps: vec![
                "1. Assess application compatibility".to_string(),
                "2. Plan migration window".to_string(),
                format!("3. Set up equivalent infrastructure on {}", provider),
                "4. Migrate data and test".to_string(),
                "5. Switch traffic and decommission old resources".to_string(),
            ],
        })
    }
}
```

#### 3.2 Recommendation Aggregator

```rust
// libs/domains/finops/src/optimizer/mod.rs

pub struct OptimizationEngine {
    rules: Vec<Box<dyn OptimizationRule>>,
    pricing_service: Arc<PricingService>,
}

impl OptimizationEngine {
    pub fn new(pricing_service: Arc<PricingService>) -> Self {
        Self {
            rules: vec![
                Box::new(UnderutilizedComputeRule::default()),
                Box::new(UnattachedStorageRule),
                Box::new(ReservedInstanceRule::default()),
                Box::new(CrossProviderMigrationRule::default()),
                Box::new(OldGenerationInstanceRule),
                Box::new(OverprovisionedDatabaseRule),
                Box::new(IdleLoadBalancerRule),
                Box::new(UnusedElasticIpRule),
            ],
            pricing_service,
        }
    }

    pub async fn analyze_resources(
        &self,
        resources: &[CloudResource],
    ) -> Vec<Recommendation> {
        let context = OptContext::new(self.pricing_service.clone()).await;

        let mut recommendations = Vec::new();

        for resource in resources {
            for rule in &self.rules {
                if rule.applies_to(resource) {
                    if let Some(rec) = rule.analyze(resource, &context) {
                        recommendations.push(rec);
                    }
                }
            }
        }

        // Sort by potential savings
        recommendations.sort_by(|a, b| b.savings_cents.cmp(&a.savings_cents));

        recommendations
    }

    pub fn generate_summary(&self, recommendations: &[Recommendation]) -> OptimizationSummary {
        let total_savings: i64 = recommendations.iter().map(|r| r.savings_cents).sum();

        let by_type = recommendations
            .iter()
            .fold(HashMap::new(), |mut acc, r| {
                let entry = acc.entry(r.recommendation_type.clone()).or_insert((0, 0i64));
                entry.0 += 1;
                entry.1 += r.savings_cents;
                acc
            });

        OptimizationSummary {
            total_recommendations: recommendations.len(),
            total_monthly_savings_cents: total_savings,
            total_annual_savings_cents: total_savings * 12,
            by_type,
            top_recommendations: recommendations.iter().take(5).cloned().collect(),
            confidence_weighted_savings: self.calculate_weighted_savings(recommendations),
        }
    }
}
```

---

### Phase 4: Multi-Agent Architecture (Weeks 7-8)

**Goal**: Specialized agents communicating via gRPC

#### 4.1 Agent Proto Definitions

```protobuf
// libs/protos/finops/v1/agents.proto
syntax = "proto3";

package finops.v1;

import "google/protobuf/struct.proto";

// Agent service for inter-agent communication
service FinopsAgentService {
  // Get agent capabilities
  rpc GetCapabilities(GetCapabilitiesRequest) returns (AgentCapabilities);

  // Execute a task
  rpc ExecuteTask(TaskRequest) returns (TaskResponse);

  // Stream task execution
  rpc ExecuteTaskStream(TaskRequest) returns (stream TaskUpdate);

  // Collaborative task (multi-agent)
  rpc CollaborateOnTask(CollaborationRequest) returns (stream CollaborationUpdate);
}

message AgentCapabilities {
  string agent_id = 1;
  string name = 2;
  string description = 3;
  repeated string skills = 4;
  repeated ToolDefinition tools = 5;
}

message ToolDefinition {
  string name = 1;
  string description = 2;
  google.protobuf.Struct input_schema = 3;
  google.protobuf.Struct output_schema = 4;
}

message TaskRequest {
  string task_id = 1;
  string from_agent = 2;
  string instruction = 3;
  google.protobuf.Struct context = 4;
  google.protobuf.Struct parameters = 5;
}

message TaskResponse {
  string task_id = 1;
  TaskStatus status = 2;
  string result = 3;
  google.protobuf.Struct structured_result = 4;
  repeated ToolCallRecord tool_calls = 5;
}

enum TaskStatus {
  TASK_STATUS_UNSPECIFIED = 0;
  TASK_STATUS_PENDING = 1;
  TASK_STATUS_RUNNING = 2;
  TASK_STATUS_COMPLETED = 3;
  TASK_STATUS_FAILED = 4;
  TASK_STATUS_NEEDS_INPUT = 5;
}

message TaskUpdate {
  string task_id = 1;
  oneof update {
    string progress = 2;
    string thought = 3;
    ToolCallRecord tool_call = 4;
    TaskResponse result = 5;
  }
}

message ToolCallRecord {
  string tool_name = 1;
  string arguments = 2;
  string result = 3;
  int64 latency_ms = 4;
}

message CollaborationRequest {
  string collaboration_id = 1;
  string initiator_agent = 2;
  repeated string participant_agents = 3;
  string goal = 4;
  google.protobuf.Struct shared_context = 5;
}

message CollaborationUpdate {
  string collaboration_id = 1;
  string from_agent = 2;
  oneof update {
    string message = 3;
    TaskRequest task_delegation = 4;
    TaskResponse task_result = 5;
    string conclusion = 6;
  }
}
```

#### 4.2 Specialized Agents

```rust
// apps/zerg/finops-pricing-agent/src/main.rs
// Specialized agent for pricing queries

use tonic::{Request, Response, Status};
use rig::{agent::Agent, providers::openai::Client};

pub struct PricingAgent {
    agent: Agent,
    service: Arc<PricingService>,
}

#[tonic::async_trait]
impl FinopsAgentService for PricingAgent {
    async fn get_capabilities(
        &self,
        _request: Request<GetCapabilitiesRequest>,
    ) -> Result<Response<AgentCapabilities>, Status> {
        Ok(Response::new(AgentCapabilities {
            agent_id: "pricing-agent-v1".into(),
            name: "Pricing Specialist".into(),
            description: "Expert in cloud pricing, comparisons, and cost calculations".into(),
            skills: vec![
                "price_comparison".into(),
                "tco_calculation".into(),
                "reserved_instance_analysis".into(),
                "spot_pricing_analysis".into(),
            ],
            tools: self.get_tool_definitions(),
        }))
    }

    type ExecuteTaskStreamStream = ReceiverStream<Result<TaskUpdate, Status>>;

    async fn execute_task_stream(
        &self,
        request: Request<TaskRequest>,
    ) -> Result<Response<Self::ExecuteTaskStreamStream>, Status> {
        let task = request.into_inner();
        let (tx, rx) = mpsc::channel(32);

        let agent = self.agent.clone();
        tokio::spawn(async move {
            // Stream agent execution
            let mut stream = agent.prompt_stream(&task.instruction).await.unwrap();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    AgentChunk::Text(text) => {
                        tx.send(Ok(TaskUpdate {
                            task_id: task.task_id.clone(),
                            update: Some(Update::Progress(text)),
                        })).await.ok();
                    }
                    AgentChunk::ToolCall { name, args, result } => {
                        tx.send(Ok(TaskUpdate {
                            task_id: task.task_id.clone(),
                            update: Some(Update::ToolCall(ToolCallRecord {
                                tool_name: name,
                                arguments: args,
                                result,
                                latency_ms: 0,
                            })),
                        })).await.ok();
                    }
                    AgentChunk::Done(result) => {
                        tx.send(Ok(TaskUpdate {
                            task_id: task.task_id.clone(),
                            update: Some(Update::Result(TaskResponse {
                                task_id: task.task_id.clone(),
                                status: TaskStatus::Completed.into(),
                                result,
                                ..Default::default()
                            })),
                        })).await.ok();
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

#### 4.3 Orchestrator with Agent Routing

```rust
// libs/domains/finops/src/agent/orchestrator.rs

pub struct MultiAgentOrchestrator {
    // gRPC clients to specialized agents
    pricing_agent: FinopsAgentServiceClient<Channel>,
    resource_agent: FinopsAgentServiceClient<Channel>,
    optimizer_agent: FinopsAgentServiceClient<Channel>,

    // Local LLM for routing decisions
    router: Agent,
}

impl MultiAgentOrchestrator {
    pub async fn handle_request(
        &self,
        session: &ChatSession,
        message: &str,
    ) -> Result<ChatResponse> {
        // 1. Classify intent and route to appropriate agent(s)
        let routing = self.router
            .prompt(&format!(
                "Classify this FinOps request and determine which agents should handle it:\n\n\
                 Request: {}\n\n\
                 Available agents:\n\
                 - pricing: Price comparisons, TCO, cost calculations\n\
                 - resource: Cloud resource exploration, inventory\n\
                 - optimizer: Recommendations, rightsizing, migrations\n\n\
                 Respond with JSON: {{\"agents\": [\"agent1\", \"agent2\"], \"strategy\": \"sequential|parallel\"}}",
                message
            ))
            .await?;

        let routing: RoutingDecision = serde_json::from_str(&routing)?;

        // 2. Execute based on strategy
        match routing.strategy.as_str() {
            "parallel" => self.execute_parallel(&routing.agents, message, session).await,
            "sequential" => self.execute_sequential(&routing.agents, message, session).await,
            _ => self.execute_sequential(&routing.agents, message, session).await,
        }
    }

    async fn execute_parallel(
        &self,
        agents: &[String],
        message: &str,
        session: &ChatSession,
    ) -> Result<ChatResponse> {
        let task_id = Uuid::new_v4().to_string();
        let mut handles = Vec::new();

        for agent_name in agents {
            let client = self.get_client(agent_name)?;
            let request = TaskRequest {
                task_id: task_id.clone(),
                from_agent: "orchestrator".into(),
                instruction: message.to_string(),
                context: session.context.clone(),
                parameters: Default::default(),
            };

            handles.push(tokio::spawn(async move {
                client.execute_task(request).await
            }));
        }

        // Collect results
        let results = futures::future::join_all(handles).await;

        // Synthesize responses
        self.synthesize_responses(results).await
    }

    async fn execute_sequential(
        &self,
        agents: &[String],
        message: &str,
        session: &ChatSession,
    ) -> Result<ChatResponse> {
        let mut context = session.context.clone();
        let mut final_response = String::new();

        for agent_name in agents {
            let client = self.get_client(agent_name)?;

            let response = client
                .execute_task(TaskRequest {
                    task_id: Uuid::new_v4().to_string(),
                    from_agent: "orchestrator".into(),
                    instruction: message.to_string(),
                    context: context.clone(),
                    parameters: Default::default(),
                })
                .await?
                .into_inner();

            // Add result to context for next agent
            context.insert(
                format!("{}_result", agent_name),
                response.result.clone().into(),
            );

            final_response = response.result;
        }

        Ok(ChatResponse {
            content: final_response,
            tool_calls: vec![],
        })
    }
}
```

---

### Phase 5: Write Operations (Weeks 9-10)

**Goal**: Implement approved actions with safety controls

#### 5.1 Action Approval Workflow

```rust
// libs/domains/finops/src/actions/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinopsAction {
    // Read-only (Phase 1)
    ExploreResources(ExploreParams),
    ComparePrice(CompareParams),

    // Write with approval (Phase 5)
    RightsizeInstance(RightsizeParams),
    TerminateResource(TerminateParams),
    CreateReservedInstance(ReservedParams),
    MigrateResource(MigrateParams),
}

impl FinopsAction {
    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            Self::RightsizeInstance(_)
                | Self::TerminateResource(_)
                | Self::CreateReservedInstance(_)
                | Self::MigrateResource(_)
        )
    }

    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Self::ExploreResources(_) | Self::ComparePrice(_) => RiskLevel::None,
            Self::RightsizeInstance(_) => RiskLevel::Medium,
            Self::CreateReservedInstance(_) => RiskLevel::Medium,
            Self::TerminateResource(_) => RiskLevel::High,
            Self::MigrateResource(_) => RiskLevel::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub action: FinopsAction,
    pub status: ActionStatus,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub executed_at: Option<DateTime<Utc>>,
    pub result: Option<ActionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    PendingApproval,
    Approved,
    Rejected,
    Executing,
    Completed,
    Failed,
    RolledBack,
}

// Action executor with safety checks
pub struct ActionExecutor {
    aws: AwsExecutor,
    azure: AzureExecutor,
    gcp: GcpExecutor,
    db: DatabasePool,
}

impl ActionExecutor {
    pub async fn execute(&self, request: &ActionRequest) -> Result<ActionResult> {
        // Verify approval
        if request.action.requires_approval() && request.status != ActionStatus::Approved {
            return Err(FinopsError::NotApproved);
        }

        // Create rollback point
        let rollback_info = self.create_rollback_point(&request.action).await?;

        // Execute with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(300),
            self.execute_action(&request.action),
        )
        .await
        .map_err(|_| FinopsError::Timeout)?;

        match result {
            Ok(result) => {
                self.cleanup_rollback_point(&rollback_info).await?;
                Ok(result)
            }
            Err(e) => {
                // Attempt rollback
                if let Err(rollback_err) = self.rollback(&rollback_info).await {
                    tracing::error!("Rollback failed: {:?}", rollback_err);
                }
                Err(e)
            }
        }
    }

    async fn execute_action(&self, action: &FinopsAction) -> Result<ActionResult> {
        match action {
            FinopsAction::RightsizeInstance(params) => {
                match params.provider.as_str() {
                    "aws" => self.aws.rightsize_instance(params).await,
                    "azure" => self.azure.rightsize_instance(params).await,
                    "gcp" => self.gcp.rightsize_instance(params).await,
                    _ => Err(FinopsError::UnsupportedProvider),
                }
            }
            FinopsAction::TerminateResource(params) => {
                // Double-check confirmation
                if !params.confirmed {
                    return Err(FinopsError::NotConfirmed);
                }
                // Execute termination
                self.terminate_resource(params).await
            }
            // ... other actions
            _ => Err(FinopsError::UnsupportedAction),
        }
    }
}
```

#### 5.2 Approval UI Component

```typescript
// apps/cloud/web/src/components/action-approval.tsx
import { createSignal, Show } from 'solid-js';

interface ActionApprovalProps {
  action: FinopsAction;
  onApprove: (actionId: string) => Promise<void>;
  onReject: (actionId: string, reason: string) => Promise<void>;
}

export function ActionApproval(props: ActionApprovalProps) {
  const [confirming, setConfirming] = createSignal(false);
  const [rejectReason, setRejectReason] = createSignal('');

  const riskColors = {
    none: 'bg-green-100 text-green-800',
    medium: 'bg-yellow-100 text-yellow-800',
    high: 'bg-red-100 text-red-800',
  };

  return (
    <div class="border rounded-lg p-4 bg-white shadow">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold">{props.action.title}</h3>
        <span class={`px-2 py-1 rounded text-sm ${riskColors[props.action.riskLevel]}`}>
          {props.action.riskLevel} risk
        </span>
      </div>

      <p class="text-gray-600 mb-4">{props.action.description}</p>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div>
          <span class="text-sm text-gray-500">Current Cost</span>
          <p class="font-medium">${props.action.currentCost}/mo</p>
        </div>
        <div>
          <span class="text-sm text-gray-500">Projected Cost</span>
          <p class="font-medium text-green-600">${props.action.projectedCost}/mo</p>
        </div>
      </div>

      <div class="mb-4">
        <span class="text-sm text-gray-500">Estimated Savings</span>
        <p class="text-xl font-bold text-green-600">
          ${props.action.savings}/mo (${props.action.savings * 12}/year)
        </p>
      </div>

      <Show when={props.action.risks.length > 0}>
        <div class="mb-4">
          <span class="text-sm text-gray-500">Risks</span>
          <ul class="list-disc list-inside text-sm text-red-600">
            {props.action.risks.map(risk => <li>{risk}</li>)}
          </ul>
        </div>
      </Show>

      <Show when={!confirming()}>
        <div class="flex gap-2">
          <button
            class="flex-1 bg-green-600 text-white py-2 rounded hover:bg-green-700"
            onClick={() => setConfirming(true)}
          >
            Approve
          </button>
          <button
            class="flex-1 bg-gray-200 text-gray-800 py-2 rounded hover:bg-gray-300"
            onClick={() => props.onReject(props.action.id, rejectReason())}
          >
            Reject
          </button>
        </div>
      </Show>

      <Show when={confirming()}>
        <div class="border-t pt-4">
          <p class="text-red-600 font-medium mb-2">
            Are you sure? This action will modify your cloud infrastructure.
          </p>
          <div class="flex gap-2">
            <button
              class="flex-1 bg-red-600 text-white py-2 rounded hover:bg-red-700"
              onClick={() => props.onApprove(props.action.id)}
            >
              Yes, Execute
            </button>
            <button
              class="flex-1 bg-gray-200 text-gray-800 py-2 rounded hover:bg-gray-300"
              onClick={() => setConfirming(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}
```

---

## Frontend Implementation

### Chat Component with TanStack AI

```typescript
// apps/cloud/web/src/components/finops-chat.tsx
import { createChat } from '@tanstack/ai-solid';
import { createSignal, For, Show } from 'solid-js';

export function FinopsChat() {
  const [sessionId, setSessionId] = createSignal<string | null>(null);

  const chat = createChat({
    api: '/api/finops/chat/stream',
    streamProtocol: 'sse',
    body: () => ({
      session_id: sessionId(),
    }),
    onResponse: (response) => {
      // Extract session ID from first response
      const newSessionId = response.headers.get('x-session-id');
      if (newSessionId && !sessionId()) {
        setSessionId(newSessionId);
      }
    },
  });

  const [input, setInput] = createSignal('');

  const suggestions = [
    "Compare compute prices across AWS, Azure, and GCP",
    "What's the cheapest way to run a PostgreSQL database?",
    "Analyze my AWS resources for optimization opportunities",
    "Calculate TCO for self-managed Kubernetes vs EKS",
  ];

  return (
    <div class="flex flex-col h-full">
      {/* Messages */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <Show when={chat.messages().length === 0}>
          <div class="text-center py-8">
            <h2 class="text-xl font-semibold mb-4">FinOps Assistant</h2>
            <p class="text-gray-600 mb-6">
              I can help you optimize cloud costs, compare providers, and find savings.
            </p>
            <div class="grid grid-cols-2 gap-2">
              {suggestions.map(suggestion => (
                <button
                  class="text-left p-3 border rounded-lg hover:bg-gray-50 text-sm"
                  onClick={() => {
                    setInput(suggestion);
                    chat.send(suggestion);
                  }}
                >
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        </Show>

        <For each={chat.messages()}>
          {(message) => (
            <div class={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}>
              <div class={`max-w-[80%] rounded-lg p-3 ${
                message.role === 'user'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-900'
              }`}>
                <Show when={message.role === 'assistant'}>
                  <MessageContent content={message.content} />
                </Show>
                <Show when={message.role === 'user'}>
                  {message.content}
                </Show>
              </div>
            </div>
          )}
        </For>

        <Show when={chat.isLoading()}>
          <div class="flex justify-start">
            <div class="bg-gray-100 rounded-lg p-3">
              <div class="flex items-center gap-2">
                <div class="animate-spin h-4 w-4 border-2 border-blue-600 border-t-transparent rounded-full" />
                <span class="text-gray-600">Analyzing...</span>
              </div>
            </div>
          </div>
        </Show>
      </div>

      {/* Input */}
      <div class="border-t p-4">
        <form
          class="flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (input().trim()) {
              chat.send(input());
              setInput('');
            }
          }}
        >
          <input
            type="text"
            value={input()}
            onInput={(e) => setInput(e.target.value)}
            placeholder="Ask about cloud costs, optimizations, or comparisons..."
            class="flex-1 border rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            disabled={chat.isLoading() || !input().trim()}
            class="bg-blue-600 text-white px-6 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            Send
          </button>
        </form>
      </div>
    </div>
  );
}

// Render markdown with tables and code blocks
function MessageContent(props: { content: string }) {
  // Use a markdown renderer like solid-markdown
  return <div class="prose prose-sm" innerHTML={renderMarkdown(props.content)} />;
}
```

---

## Deployment Architecture

```yaml
# manifests/kustomize/finops/base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: finops-orchestrator
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: orchestrator
          image: finops-orchestrator:latest
          ports:
            - containerPort: 8080  # HTTP
            - containerPort: 50051 # gRPC
          env:
            - name: OPENAI_API_KEY
              valueFrom:
                secretKeyRef:
                  name: finops-secrets
                  key: openai-api-key
            - name: PRICING_AGENT_ADDR
              value: "finops-pricing-agent:50051"
            - name: RESOURCE_AGENT_ADDR
              value: "finops-resource-agent:50051"
            - name: OPTIMIZER_AGENT_ADDR
              value: "finops-optimizer-agent:50051"
          resources:
            requests:
              memory: "512Mi"
              cpu: "250m"
            limits:
              memory: "2Gi"
              cpu: "1000m"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: finops-pricing-agent
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: agent
          image: finops-pricing-agent:latest
          ports:
            - containerPort: 50051
          resources:
            requests:
              memory: "256Mi"
              cpu: "100m"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: finops-sync-worker
spec:
  replicas: 1
  template:
    spec:
      containers:
        - name: worker
          image: finops-sync-worker:latest
          env:
            - name: REDIS_URL
              value: "redis://redis:6379"
```

---

## Summary

### Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **Phase 1** | Weeks 1-2 | Chat UI, basic agent, pricing tools |
| **Phase 2** | Weeks 3-4 | Cloud SDK integration, resource sync |
| **Phase 3** | Weeks 5-6 | Optimization engine, recommendations |
| **Phase 4** | Weeks 7-8 | Multi-agent gRPC architecture |
| **Phase 5** | Weeks 9-10 | Write operations with approval workflow |

### Key Technologies

| Component | Technology |
|-----------|------------|
| Frontend Chat | TanStack AI + SolidJS |
| Backend Agent | Rig (Rust LLM framework) |
| Agent Communication | gRPC (existing infrastructure) |
| Async Processing | Redis Streams (existing) |
| Cloud APIs | AWS/Azure/GCP SDKs |
| LLM Provider | OpenAI GPT-4o (configurable) |

### Architecture Decisions

1. **Rust-native agents** - Use Rig instead of Python for consistency with your stack
2. **gRPC for agents** - Leverage existing infrastructure for agent-to-agent communication
3. **Streaming responses** - SSE for chat, gRPC streams for long operations
4. **Phased write operations** - Read-only first, then approved actions
5. **Multi-agent pattern** - Specialized agents for pricing, resources, optimization
