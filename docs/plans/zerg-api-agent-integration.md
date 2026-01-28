# Plan: Agent Integration in zerg-api

Add agent invocation capabilities to `apps/zerg/api`, consolidating the functionality currently in `apps/agents/gateway`.

## Problem

Current architecture has **duplicated infrastructure**:

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│   Client    │────▶│  agent-gateway   │────▶│   Agents    │
│  (TanStack) │     │   (Express.js)   │     │ (LangGraph) │
└─────────────┘     └──────────────────┘     └─────────────┘
                           │
                           ├─ Auth middleware
                           ├─ Rate limiting
                           ├─ Health checks
                           └─ Tracing (Braintrust)

┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│   Client    │────▶│     zerg-api     │────▶│ gRPC/Direct │
│  (TanStack) │     │    (Rust Axum)   │     │  Services   │
└─────────────┘     └──────────────────┘     └─────────────┘
                           │
                           ├─ Auth middleware
                           ├─ Rate limiting
                           ├─ Health checks
                           └─ Tracing (OpenTelemetry)
```

**Issues:**
1. Two separate services with overlapping concerns
2. Different auth/tracing implementations
3. Separate deployment and monitoring
4. Clients need two different base URLs

## Solution

Consolidate agent routes into zerg-api with three communication patterns:

```
┌─────────────┐     ┌──────────────────────────────────────────────┐
│   Client    │────▶│                  zerg-api                    │
│  (TanStack) │     │                (Rust Axum)                   │
└─────────────┘     │                                              │
                    │  /api/tasks/*        → gRPC to zerg-tasks    │
                    │  /api/vectors/*      → gRPC to zerg-vector   │
                    │  /api/agents/*       → HTTP/NATS/gRPC        │
                    └───────────┬──────────────────────────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   ┌────────────┐       ┌────────────┐       ┌────────────┐
   │  HTTP Call │       │   NATS     │       │   gRPC     │
   │ (LangServe)│       │  Pub/Sub   │       │  (Tonic)   │
   └────────────┘       └────────────┘       └────────────┘
          │                     │                     │
          ▼                     ▼                     ▼
   ┌────────────┐       ┌────────────┐       ┌────────────┐
   │ rag-agent  │       │code-tester │       │ supervisor │
   │(LangGraph) │       │(LangGraph) │       │(LangGraph) │
   └────────────┘       └────────────┘       └────────────┘
```

## Existing Agent Inventory

| Agent | Description | Current Exposure | Recommended Pattern |
|-------|-------------|------------------|---------------------|
| `rag-agent` | RAG retrieval graph | None (graph only) | HTTP (LangServe) |
| `code-tester` | Memory-enhanced agent | None (graph only) | HTTP (LangServe) |
| `whatsup-agent` | Tools agent with WhatsApp integration | None (graph only) | NATS (async) |
| `supervisor-langgraph` | Multi-agent orchestrator | None (graph only) | gRPC (streaming) |

## API Design

### Routes Structure

```
/api/agents
├── GET  /                     # List all registered agents
├── GET  /:name               # Get agent info
├── GET  /:name/health        # Check agent health
├── POST /:name/invoke        # Invoke agent (sync)
├── POST /:name/stream        # Stream from agent (SSE)
└── POST /:name/converse      # Bidirectional conversation
```

### Request Schema

```rust
// apps/zerg/api/src/api/agents.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InvokeRequest {
    pub messages: Vec<AgentMessage>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvokeResponse {
    pub request_id: String,
    pub agent_name: String,
    pub messages: Vec<AgentMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub protocol: AgentProtocol,
    pub tags: Vec<String>,
    pub health: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AgentProtocol {
    Http,
    Nats,
    Grpc,
}
```

## Implementation: Three Patterns

### Pattern 1: HTTP (for rag-agent, code-tester)

Uses LangServe's `/invoke` and `/stream` endpoints directly.

```rust
// apps/zerg/api/src/api/agents/http.rs

use reqwest::Client;
use crate::api::agents::{InvokeRequest, InvokeResponse, AgentMessage};

pub struct HttpAgentClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl HttpAgentClient {
    pub fn new(base_url: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            timeout,
        }
    }

    /// Invoke agent synchronously
    pub async fn invoke(
        &self,
        request: InvokeRequest,
        trace_id: &str,
    ) -> Result<InvokeResponse, AgentError> {
        let url = format!("{}/invoke", self.base_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Trace-Id", trace_id)
            .json(&serde_json::json!({
                "input": {
                    "messages": request.messages,
                },
                "config": request.config,
                "metadata": request.metadata,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AgentError::InvocationFailed(error_text));
        }

        let result: serde_json::Value = response.json().await?;

        // Parse LangServe response format
        let output = result.get("output").ok_or(AgentError::InvalidResponse)?;
        let messages = parse_langgraph_output(output)?;

        Ok(InvokeResponse {
            request_id: trace_id.to_string(),
            agent_name: "rag-agent".to_string(),
            messages,
            metadata: None,
        })
    }

    /// Stream from agent via SSE
    pub async fn stream(
        &self,
        request: InvokeRequest,
        trace_id: &str,
    ) -> Result<impl Stream<Item = Result<StreamChunk, AgentError>>, AgentError> {
        let url = format!("{}/stream", self.base_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("X-Trace-Id", trace_id)
            .json(&serde_json::json!({
                "input": { "messages": request.messages },
                "config": request.config,
            }))
            .send()
            .await?;

        Ok(parse_sse_stream(response.bytes_stream()))
    }
}

fn parse_langgraph_output(output: &serde_json::Value) -> Result<Vec<AgentMessage>, AgentError> {
    // LangGraph output format: { "messages": [...] }
    let messages = output
        .get("messages")
        .and_then(|m| m.as_array())
        .ok_or(AgentError::InvalidResponse)?;

    messages
        .iter()
        .filter_map(|msg| {
            let role = msg.get("type")?.as_str()?;
            let content = msg.get("content")?.as_str()?;
            Some(AgentMessage {
                role: match role {
                    "human" => MessageRole::User,
                    "ai" => MessageRole::Assistant,
                    "system" => MessageRole::System,
                    _ => return None,
                },
                content: content.to_string(),
            })
        })
        .collect::<Vec<_>>()
        .pipe(Ok)
}
```

### Pattern 2: NATS (for whatsup-agent, async workflows)

For fire-and-forget or long-running agent tasks.

```rust
// apps/zerg/api/src/api/agents/nats.rs

use async_nats::{Client as NatsClient, jetstream};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct NatsAgentMessage {
    request_id: String,
    reply_to: String,
    payload: InvokeRequest,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct NatsAgentResponse {
    request_id: String,
    payload: AgentResult,
    timestamp: i64,
}

pub struct NatsAgentClient {
    client: NatsClient,
    timeout: Duration,
}

impl NatsAgentClient {
    pub async fn new(nats_url: &str) -> Result<Self, AgentError> {
        let client = async_nats::connect(nats_url).await?;
        Ok(Self {
            client,
            timeout: Duration::from_secs(30),
        })
    }

    /// Request-reply pattern for synchronous invocation
    pub async fn invoke(
        &self,
        agent_name: &str,
        request: InvokeRequest,
        trace_id: &str,
    ) -> Result<InvokeResponse, AgentError> {
        let subject = format!("agents.{}.request", agent_name);
        let request_id = Uuid::new_v4().to_string();

        let message = NatsAgentMessage {
            request_id: request_id.clone(),
            reply_to: format!("agents.{}.response.{}", agent_name, request_id),
            payload: request,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let payload = serde_json::to_vec(&message)?;

        // Use NATS request-reply pattern
        let response = tokio::time::timeout(
            self.timeout,
            self.client.request(subject, payload.into())
        )
        .await
        .map_err(|_| AgentError::Timeout)?
        .map_err(AgentError::NatsError)?;

        let nats_response: NatsAgentResponse = serde_json::from_slice(&response.payload)?;

        Ok(InvokeResponse {
            request_id: nats_response.request_id,
            agent_name: agent_name.to_string(),
            messages: nats_response.payload.messages,
            metadata: None,
        })
    }

    /// Fire-and-forget pattern for async tasks
    pub async fn invoke_async(
        &self,
        agent_name: &str,
        request: InvokeRequest,
        callback_url: Option<String>,
    ) -> Result<String, AgentError> {
        let subject = format!("agents.{}.request", agent_name);
        let request_id = Uuid::new_v4().to_string();

        let message = serde_json::json!({
            "request_id": request_id,
            "payload": request,
            "callback_url": callback_url,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        let payload = serde_json::to_vec(&message)?;
        self.client.publish(subject, payload.into()).await?;

        Ok(request_id) // Return request ID for tracking
    }
}
```

### Pattern 3: gRPC (for supervisor, high-performance streaming)

For bidirectional streaming and complex orchestration.

```rust
// apps/zerg/api/src/api/agents/grpc.rs

use tonic::transport::Channel;
use proto::agent_service_client::AgentServiceClient;
use tokio_stream::StreamExt;

pub struct GrpcAgentClient {
    client: AgentServiceClient<Channel>,
}

impl GrpcAgentClient {
    pub async fn new(addr: &str) -> Result<Self, AgentError> {
        let channel = Channel::from_shared(addr.to_string())?
            .connect_lazy();

        Ok(Self {
            client: AgentServiceClient::new(channel)
                .accept_compressed(CompressionEncoding::Zstd)
                .send_compressed(CompressionEncoding::Zstd),
        })
    }

    /// Unary invoke
    pub async fn invoke(
        &mut self,
        request: InvokeRequest,
        trace_id: &str,
    ) -> Result<InvokeResponse, AgentError> {
        let proto_request = proto::InvokeRequest {
            messages: request.messages.into_iter().map(Into::into).collect(),
            config: request.config.map(|c| c.to_string()),
            metadata: Some(proto::Metadata {
                trace_id: trace_id.to_string(),
                ..Default::default()
            }),
        };

        let response = self.client.invoke(proto_request).await?;
        let inner = response.into_inner();

        Ok(InvokeResponse {
            request_id: inner.request_id,
            agent_name: inner.agent_name,
            messages: inner.messages.into_iter().map(Into::into).collect(),
            metadata: None,
        })
    }

    /// Server streaming
    pub async fn stream(
        &mut self,
        request: InvokeRequest,
        trace_id: &str,
    ) -> Result<impl Stream<Item = Result<StreamChunk, AgentError>>, AgentError> {
        let proto_request = proto::InvokeRequest {
            messages: request.messages.into_iter().map(Into::into).collect(),
            config: request.config.map(|c| c.to_string()),
            metadata: Some(proto::Metadata {
                trace_id: trace_id.to_string(),
                ..Default::default()
            }),
        };

        let response = self.client.stream(proto_request).await?;
        let stream = response.into_inner();

        Ok(stream.map(|result| {
            result
                .map(|chunk| StreamChunk {
                    content: chunk.content,
                    done: chunk.done,
                })
                .map_err(AgentError::GrpcError)
        }))
    }

    /// Bidirectional streaming for conversations
    pub async fn converse(
        &mut self,
        input: impl Stream<Item = AgentMessage> + Send + 'static,
    ) -> Result<impl Stream<Item = Result<AgentMessage, AgentError>>, AgentError> {
        let request_stream = input.map(|msg| proto::Message {
            role: msg.role.into(),
            content: msg.content,
        });

        let response = self.client.converse(request_stream).await?;
        let stream = response.into_inner();

        Ok(stream.map(|result| {
            result
                .map(|msg| AgentMessage {
                    role: msg.role.into(),
                    content: msg.content,
                })
                .map_err(AgentError::GrpcError)
        }))
    }
}
```

## Agent Registry

Centralized registry to manage agent configurations:

```rust
// apps/zerg/api/src/api/agents/registry.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub description: String,
    pub protocol: AgentProtocol,
    pub endpoint: String,
    pub timeout: Duration,
    pub tags: Vec<String>,
}

pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
    health_cache: Arc<RwLock<HashMap<String, HealthStatus>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            health_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize from environment/config
    pub async fn from_env() -> Result<Self, ConfigError> {
        let registry = Self::new();

        // Register agents from environment variables
        // RAG_AGENT_URL, CODE_TESTER_URL, etc.

        if let Ok(url) = std::env::var("RAG_AGENT_URL") {
            registry.register(AgentConfig {
                name: "rag-agent".to_string(),
                description: "RAG retrieval agent for document Q&A".to_string(),
                protocol: AgentProtocol::Http,
                endpoint: url,
                timeout: Duration::from_secs(30),
                tags: vec!["rag".to_string(), "retrieval".to_string()],
            }).await;
        }

        if let Ok(url) = std::env::var("WHATSUP_AGENT_NATS") {
            registry.register(AgentConfig {
                name: "whatsup-agent".to_string(),
                description: "WhatsApp integration agent".to_string(),
                protocol: AgentProtocol::Nats,
                endpoint: url,
                timeout: Duration::from_secs(60),
                tags: vec!["whatsapp".to_string(), "messaging".to_string()],
            }).await;
        }

        if let Ok(url) = std::env::var("SUPERVISOR_GRPC_URL") {
            registry.register(AgentConfig {
                name: "supervisor".to_string(),
                description: "Multi-agent orchestrator".to_string(),
                protocol: AgentProtocol::Grpc,
                endpoint: url,
                timeout: Duration::from_secs(120),
                tags: vec!["orchestrator".to_string(), "multi-agent".to_string()],
            }).await;
        }

        Ok(registry)
    }

    pub async fn register(&self, config: AgentConfig) {
        let mut agents = self.agents.write().await;
        agents.insert(config.name.clone(), config);
    }

    pub async fn get(&self, name: &str) -> Option<AgentConfig> {
        let agents = self.agents.read().await;
        agents.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<AgentConfig> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Periodic health check (runs in background)
    pub async fn health_check_loop(&self, interval: Duration) {
        loop {
            let agents = self.list().await;
            for agent in agents {
                let health = self.check_agent_health(&agent).await;
                let mut cache = self.health_cache.write().await;
                cache.insert(agent.name.clone(), health);
            }
            tokio::time::sleep(interval).await;
        }
    }

    async fn check_agent_health(&self, agent: &AgentConfig) -> HealthStatus {
        match agent.protocol {
            AgentProtocol::Http => {
                let url = format!("{}/health", agent.endpoint);
                match reqwest::get(&url).await {
                    Ok(resp) if resp.status().is_success() => HealthStatus::Healthy,
                    _ => HealthStatus::Unhealthy,
                }
            }
            AgentProtocol::Nats => {
                // NATS doesn't have direct health check, check connection
                HealthStatus::Unknown
            }
            AgentProtocol::Grpc => {
                // Use gRPC health protocol
                HealthStatus::Unknown // Implement grpc health check
            }
        }
    }
}
```

## Axum Route Handlers

```rust
// apps/zerg/api/src/api/agents/handlers.rs

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;

pub fn routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_agents))
        .route("/:name", get(get_agent))
        .route("/:name/health", get(check_health))
        .route("/:name/invoke", post(invoke_agent))
        .route("/:name/stream", post(stream_agent))
        .with_state(state.clone())
}

/// GET /api/agents
#[utoipa::path(
    get,
    path = "/api/agents",
    responses((status = 200, body = Vec<AgentInfo>)),
    tag = "agents"
)]
async fn list_agents(State(state): State<AppState>) -> Json<Vec<AgentInfo>> {
    let agents = state.agent_registry.list().await;
    let infos: Vec<AgentInfo> = agents
        .into_iter()
        .map(|a| AgentInfo {
            name: a.name,
            description: a.description,
            protocol: a.protocol,
            tags: a.tags,
            health: state.agent_registry.get_cached_health(&a.name),
        })
        .collect();
    Json(infos)
}

/// GET /api/agents/:name
async fn get_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<AgentInfo>, ApiError> {
    let agent = state.agent_registry.get(&name).await
        .ok_or(ApiError::NotFound(format!("Agent '{}' not found", name)))?;

    Ok(Json(AgentInfo {
        name: agent.name,
        description: agent.description,
        protocol: agent.protocol,
        tags: agent.tags,
        health: state.agent_registry.get_cached_health(&name),
    }))
}

/// POST /api/agents/:name/invoke
#[utoipa::path(
    post,
    path = "/api/agents/{name}/invoke",
    request_body = InvokeRequest,
    responses((status = 200, body = InvokeResponse)),
    params(("name" = String, Path, description = "Agent name")),
    tag = "agents"
)]
async fn invoke_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<InvokeRequest>,
) -> Result<Json<InvokeResponse>, ApiError> {
    let agent = state.agent_registry.get(&name).await
        .ok_or(ApiError::NotFound(format!("Agent '{}' not found", name)))?;

    let trace_id = uuid::Uuid::new_v4().to_string();

    let response = match agent.protocol {
        AgentProtocol::Http => {
            let client = HttpAgentClient::new(&agent.endpoint, agent.timeout);
            client.invoke(request, &trace_id).await?
        }
        AgentProtocol::Nats => {
            let client = state.nats_client.as_ref()
                .ok_or(ApiError::ServiceUnavailable("NATS not configured".into()))?;
            client.invoke(&name, request, &trace_id).await?
        }
        AgentProtocol::Grpc => {
            let mut client = GrpcAgentClient::new(&agent.endpoint).await?;
            client.invoke(request, &trace_id).await?
        }
    };

    Ok(Json(response))
}

/// POST /api/agents/:name/stream
async fn stream_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<InvokeRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, ApiError> {
    let agent = state.agent_registry.get(&name).await
        .ok_or(ApiError::NotFound(format!("Agent '{}' not found", name)))?;

    let trace_id = uuid::Uuid::new_v4().to_string();

    let stream = match agent.protocol {
        AgentProtocol::Http => {
            let client = HttpAgentClient::new(&agent.endpoint, agent.timeout);
            client.stream(request, &trace_id).await?
        }
        AgentProtocol::Grpc => {
            let mut client = GrpcAgentClient::new(&agent.endpoint).await?;
            client.stream(request, &trace_id).await?
        }
        AgentProtocol::Nats => {
            return Err(ApiError::BadRequest("NATS does not support streaming".into()));
        }
    };

    let sse_stream = stream.map(|result| {
        match result {
            Ok(chunk) => Ok(Event::default().data(serde_json::to_string(&chunk).unwrap())),
            Err(e) => Ok(Event::default().data(format!(r#"{{"error":"{}"}}"#, e))),
        }
    });

    Ok(Sse::new(sse_stream))
}
```

## AppState Updates

```rust
// apps/zerg/api/src/state.rs (updated)

pub struct AppState {
    pub config: Config,
    pub tasks_client: TasksServiceClient<Channel>,
    pub vector_client: VectorServiceClient<Channel>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub jwt_auth: JwtRedisAuth,
    // New: Agent infrastructure
    pub agent_registry: AgentRegistry,
    pub nats_client: Option<NatsAgentClient>,
}
```

## Proto Definition for gRPC Agents

```protobuf
// proto/agent.proto

syntax = "proto3";
package agent;

service AgentService {
  // Unary invoke
  rpc Invoke(InvokeRequest) returns (InvokeResponse);

  // Server streaming
  rpc Stream(InvokeRequest) returns (stream StreamChunk);

  // Bidirectional streaming
  rpc Converse(stream Message) returns (stream Message);
}

message InvokeRequest {
  repeated Message messages = 1;
  optional string config = 2;       // JSON config
  optional Metadata metadata = 3;
}

message InvokeResponse {
  string request_id = 1;
  string agent_name = 2;
  repeated Message messages = 3;
  optional string metadata = 4;     // JSON metadata
}

message Message {
  MessageRole role = 1;
  string content = 2;
}

enum MessageRole {
  USER = 0;
  ASSISTANT = 1;
  SYSTEM = 2;
}

message StreamChunk {
  string content = 1;
  bool done = 2;
  optional string metadata = 3;
}

message Metadata {
  string trace_id = 1;
  string span_id = 2;
  string user_id = 3;
}
```

## Implementation Steps

| Step | Task | Files |
|------|------|-------|
| 1 | Create agent types module | `apps/zerg/api/src/api/agents/mod.rs`, `types.rs` |
| 2 | Create HTTP agent client | `apps/zerg/api/src/api/agents/http.rs` |
| 3 | Create NATS agent client | `apps/zerg/api/src/api/agents/nats.rs` |
| 4 | Create gRPC agent client | `apps/zerg/api/src/api/agents/grpc.rs` |
| 5 | Create agent registry | `apps/zerg/api/src/api/agents/registry.rs` |
| 6 | Create route handlers | `apps/zerg/api/src/api/agents/handlers.rs` |
| 7 | Update AppState | `apps/zerg/api/src/state.rs` |
| 8 | Update main.rs | `apps/zerg/api/src/main.rs` |
| 9 | Add proto definitions | `proto/agent.proto` |
| 10 | Add Cargo dependencies | `apps/zerg/api/Cargo.toml` |
| 11 | Update OpenAPI docs | `apps/zerg/api/src/openapi.rs` |
| 12 | Deploy LangServe for agents | `apps/agents/*/src/server.ts` |
| 13 | Add K8s manifests | `apps/zerg/api/k8s/` |
| 14 | Integration tests | `apps/zerg/api/tests/agents.rs` |

## Dependencies to Add

```toml
# apps/zerg/api/Cargo.toml

[dependencies]
# Agent communication
async-nats = "0.36"                    # NATS client
reqwest = { version = "0.12", features = ["stream", "json"] }
tokio-stream = "0.1"                   # Stream utilities

# Already have
tonic = { version = "0.12", features = ["transport", "zstd"] }
```

## Environment Variables

```bash
# Agent HTTP endpoints (LangServe)
RAG_AGENT_URL=http://rag-agent:8000
CODE_TESTER_URL=http://code-tester:8000

# Agent NATS subjects
NATS_URL=nats://nats:4222
WHATSUP_AGENT_NATS=agents.whatsup

# Agent gRPC endpoints
SUPERVISOR_GRPC_URL=http://supervisor:50053

# Health check interval
AGENT_HEALTH_CHECK_INTERVAL_SECS=30
```

## Deployment Strategy

### Option A: Deprecate agent-gateway (Recommended)

1. Migrate all gateway functionality to zerg-api
2. Update frontend clients to use `/api/agents/*` instead of `agent-gateway`
3. Archive `apps/agents/gateway`

### Option B: Keep Both (Transitional)

1. Add agent routes to zerg-api
2. Keep agent-gateway for backward compatibility
3. Gradually migrate clients
4. Eventually deprecate gateway

## Agent Deployment Models

### HTTP Agents (LangServe)

Each agent runs as a standalone service with LangServe:

```typescript
// apps/agents/rag-agent/src/server.ts

import { graph } from './retrieval_graph/graph.js';
import { serve } from '@langchain/langgraph-sdk/server';

serve({
  graph,
  port: 8000,
  healthCheckPath: '/health',
});
```

### NATS Agents

Agents subscribe to NATS subjects:

```typescript
// apps/agents/whatsup-agent/src/nats-server.ts

import { NatsAgentServer } from 'agent-patterns/nats';
import { graph } from './agent.js';

const server = new NatsAgentServer({
  agentName: 'whatsup-agent',
  natsUrl: process.env.NATS_URL,
  handler: async (request) => {
    const result = await graph.invoke({
      messages: request.messages,
    });
    return { content: result.messages[result.messages.length - 1].content };
  },
});

server.start();
```

### gRPC Agents

For high-performance streaming scenarios (Rust recommended):

```rust
// apps/agents/supervisor/src/main.rs

use tonic::transport::Server;
use grpc::server::GrpcServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let service = SupervisorService::new();

    GrpcServerBuilder::new("agent.AgentService", 50053)?
        .serve(AgentServiceServer::new(service))
        .await
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registry() {
        let registry = AgentRegistry::new();
        registry.register(AgentConfig {
            name: "test-agent".to_string(),
            protocol: AgentProtocol::Http,
            endpoint: "http://localhost:8000".to_string(),
            ..Default::default()
        }).await;

        let agent = registry.get("test-agent").await;
        assert!(agent.is_some());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_invoke_http_agent() {
    // Start mock LangServe server
    let mock = MockHttpAgent::start().await;

    let client = HttpAgentClient::new(&mock.url, Duration::from_secs(5));
    let response = client.invoke(
        InvokeRequest {
            messages: vec![AgentMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            config: None,
            metadata: None,
        },
        "trace-123",
    ).await.unwrap();

    assert!(!response.messages.is_empty());
}
```

## Migration from agent-gateway

| Gateway Feature | zerg-api Equivalent |
|-----------------|---------------------|
| `GET /agents` | `GET /api/agents` |
| `GET /agents/:name` | `GET /api/agents/:name` |
| `POST /agents/:name/invoke` | `POST /api/agents/:name/invoke` |
| `POST /agents/:name/stream` | `POST /api/agents/:name/stream` |
| Express auth middleware | Axum JWT middleware (existing) |
| Braintrust tracing | OpenTelemetry (existing) |
| Rate limiting | Tower rate limiter (existing) |

## Related Files

- `apps/agents/gateway/src/routes/agents.ts` - Current gateway implementation
- `apps/agents/agent-patterns/` - Communication pattern examples
- `apps/zerg/api/src/api/mod.rs` - API routes entry point
- `docs/plans/agent-communication-patterns.md` - Pattern comparison
