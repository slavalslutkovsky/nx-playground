//! Agent API handlers
//!
//! REST endpoints for AI agent invocation.
//! Browser clients call these REST endpoints, which proxy to gRPC agent services.

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use rpc::agent::{
    AgentConfig, CheckHealthRequest, GetAgentRequest, InvokeRequest, ListAgentsRequest, Message,
    MessageRole, Metadata,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tracing::info;
use utoipa::ToSchema;

use crate::state::AppState;

/// Create the agent routes
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_agents))
        .route("/{name}", get(get_agent))
        .route("/{name}/health", get(check_health))
        .route("/{name}/invoke", post(invoke_agent))
        .route("/{name}/stream", post(stream_agent))
        .with_state(state)
}

// ============= Request/Response Types =============

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InvokeRequestBody {
    pub messages: Vec<AgentMessage>,
    #[serde(default)]
    pub config: Option<AgentConfigBody>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AgentConfigBody {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub retriever_provider: Option<String>,
    pub embedding_model: Option<String>,
    pub response_model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvokeResponseBody {
    pub request_id: String,
    pub agent_name: String,
    pub messages: Vec<AgentMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ResponseMetadataBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseMetadataBody {
    pub latency_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_docs_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AgentInfoBody {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub health: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListAgentsResponse {
    pub agents: Vec<AgentInfoBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponseBody {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StreamChunkBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    pub done: bool,
}

// ============= Handlers =============

/// List all available agents
#[utoipa::path(
    get,
    path = "/api/agents",
    responses(
        (status = 200, description = "List of agents", body = ListAgentsResponse)
    ),
    tag = "agents"
)]
pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<ListAgentsResponse>, (axum::http::StatusCode, String)> {
    let mut client = state.agent_client.clone();

    let response = client
        .list_agents(ListAgentsRequest { tag_filter: None })
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Agent service error: {}", e),
            )
        })?;

    let agents = response
        .into_inner()
        .agents
        .into_iter()
        .map(|a| AgentInfoBody {
            name: a.name,
            description: a.description,
            version: a.version,
            tags: a.tags,
            capabilities: a.capabilities,
            health: match a.health {
                Some(h) => match h.status {
                    1 => "healthy".to_string(),
                    2 => "unhealthy".to_string(),
                    _ => "unknown".to_string(),
                },
                None => "unknown".to_string(),
            },
        })
        .collect();

    Ok(Json(ListAgentsResponse { agents }))
}

/// Get agent info
#[utoipa::path(
    get,
    path = "/api/agents/{name}",
    params(
        ("name" = String, Path, description = "Agent name")
    ),
    responses(
        (status = 200, description = "Agent info", body = AgentInfoBody),
        (status = 404, description = "Agent not found")
    ),
    tag = "agents"
)]
pub async fn get_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<AgentInfoBody>, (axum::http::StatusCode, String)> {
    let mut client = state.agent_client.clone();

    let response = client
        .get_agent(GetAgentRequest {
            agent_name: name.clone(),
        })
        .await
        .map_err(|e| {
            if e.code() == tonic::Code::NotFound {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    format!("Agent '{}' not found", name),
                )
            } else {
                (
                    axum::http::StatusCode::BAD_GATEWAY,
                    format!("Agent service error: {}", e),
                )
            }
        })?;

    let agent = response.into_inner();

    Ok(Json(AgentInfoBody {
        name: agent.name,
        description: agent.description,
        version: agent.version,
        tags: agent.tags,
        capabilities: agent.capabilities,
        health: match agent.health {
            Some(h) => match h.status {
                1 => "healthy".to_string(),
                2 => "unhealthy".to_string(),
                _ => "unknown".to_string(),
            },
            None => "unknown".to_string(),
        },
    }))
}

/// Check agent health
#[utoipa::path(
    get,
    path = "/api/agents/{name}/health",
    params(
        ("name" = String, Path, description = "Agent name")
    ),
    responses(
        (status = 200, description = "Agent health", body = HealthResponseBody),
        (status = 503, description = "Agent unhealthy")
    ),
    tag = "agents"
)]
pub async fn check_health(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<HealthResponseBody>, (axum::http::StatusCode, String)> {
    let mut client = state.agent_client.clone();

    let response = client
        .check_health(CheckHealthRequest {
            agent_name: name.clone(),
        })
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Agent service error: {}", e),
            )
        })?;

    let health = response.into_inner().health;

    match health {
        Some(h) if h.status == 1 => Ok(Json(HealthResponseBody {
            status: "healthy".to_string(),
            message: h.message,
        })),
        Some(h) => Err((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            h.message.unwrap_or_else(|| "Unhealthy".to_string()),
        )),
        None => Err((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Unknown health status".to_string(),
        )),
    }
}

/// Invoke agent synchronously
#[utoipa::path(
    post,
    path = "/api/agents/{name}/invoke",
    params(
        ("name" = String, Path, description = "Agent name")
    ),
    request_body = InvokeRequestBody,
    responses(
        (status = 200, description = "Agent response", body = InvokeResponseBody),
        (status = 404, description = "Agent not found"),
        (status = 502, description = "Agent invocation failed")
    ),
    tag = "agents"
)]
pub async fn invoke_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<InvokeRequestBody>,
) -> Result<Json<InvokeResponseBody>, (axum::http::StatusCode, String)> {
    let mut client = state.agent_client.clone();
    let trace_id = uuid::Uuid::new_v4().to_string();

    info!(
        trace_id = %trace_id,
        agent = %name,
        messages = body.messages.len(),
        "Invoking agent"
    );

    let messages: Vec<Message> = body
        .messages
        .iter()
        .map(|m| Message {
            role: match m.role.as_str() {
                "user" => MessageRole::User as i32,
                "assistant" => MessageRole::Assistant as i32,
                "system" => MessageRole::System as i32,
                _ => MessageRole::User as i32,
            },
            content: m.content.clone(),
            name: None,
            tool_call_id: None,
        })
        .collect();

    let config = body.config.map(|c| AgentConfig {
        user_id: c.user_id,
        session_id: c.session_id,
        retriever_provider: c.retriever_provider,
        embedding_model: c.embedding_model,
        response_model: c.response_model,
        temperature: c.temperature,
        max_tokens: c.max_tokens,
        extra_json: None,
    });

    let request = InvokeRequest {
        agent_name: name.clone(),
        messages,
        config,
        metadata: Some(Metadata {
            trace_id: trace_id.clone(),
            span_id: None,
            parent_span_id: None,
            tags: Default::default(),
        }),
    };

    let response = client.invoke(request).await.map_err(|e| {
        (
            axum::http::StatusCode::BAD_GATEWAY,
            format!("Agent invocation failed: {}", e),
        )
    })?;

    let inner = response.into_inner();

    let messages: Vec<AgentMessage> = inner
        .messages
        .iter()
        .map(|m| AgentMessage {
            role: match m.role {
                1 => "user".to_string(),
                2 => "assistant".to_string(),
                3 => "system".to_string(),
                _ => "unknown".to_string(),
            },
            content: m.content.clone(),
        })
        .collect();

    let metadata = inner.response_metadata.map(|m| ResponseMetadataBody {
        latency_ms: m.latency_ms,
        tokens_used: m.tokens_used,
        retrieved_docs_count: m.retrieved_docs_count,
        queries: if m.queries.is_empty() {
            None
        } else {
            Some(m.queries)
        },
    });

    Ok(Json(InvokeResponseBody {
        request_id: inner.request_id,
        agent_name: inner.agent_name,
        messages,
        metadata,
    }))
}

/// Stream responses from agent (SSE)
#[utoipa::path(
    post,
    path = "/api/agents/{name}/stream",
    params(
        ("name" = String, Path, description = "Agent name")
    ),
    request_body = InvokeRequestBody,
    responses(
        (status = 200, description = "SSE stream of agent responses"),
        (status = 404, description = "Agent not found"),
        (status = 502, description = "Agent streaming failed")
    ),
    tag = "agents"
)]
pub async fn stream_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<InvokeRequestBody>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (axum::http::StatusCode, String)> {
    let mut client = state.agent_client.clone();
    let trace_id = uuid::Uuid::new_v4().to_string();

    info!(
        trace_id = %trace_id,
        agent = %name,
        messages = body.messages.len(),
        "Streaming from agent"
    );

    let messages: Vec<Message> = body
        .messages
        .iter()
        .map(|m| Message {
            role: match m.role.as_str() {
                "user" => MessageRole::User as i32,
                "assistant" => MessageRole::Assistant as i32,
                "system" => MessageRole::System as i32,
                _ => MessageRole::User as i32,
            },
            content: m.content.clone(),
            name: None,
            tool_call_id: None,
        })
        .collect();

    let config = body.config.map(|c| AgentConfig {
        user_id: c.user_id,
        session_id: c.session_id,
        retriever_provider: c.retriever_provider,
        embedding_model: c.embedding_model,
        response_model: c.response_model,
        temperature: c.temperature,
        max_tokens: c.max_tokens,
        extra_json: None,
    });

    let request = InvokeRequest {
        agent_name: name.clone(),
        messages,
        config,
        metadata: Some(Metadata {
            trace_id: trace_id.clone(),
            span_id: None,
            parent_span_id: None,
            tags: Default::default(),
        }),
    };

    let response = client.stream(request).await.map_err(|e| {
        (
            axum::http::StatusCode::BAD_GATEWAY,
            format!("Agent streaming failed: {}", e),
        )
    })?;

    let stream = response.into_inner();

    let event_stream = async_stream::stream! {
        use futures::StreamExt;
        let mut stream = stream;

        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    let body = StreamChunkBody {
                        content: match &chunk.chunk {
                            Some(rpc::agent::stream_chunk::Chunk::Content(c)) => Some(c.clone()),
                            _ => None,
                        },
                        event: match &chunk.chunk {
                            Some(rpc::agent::stream_chunk::Chunk::Event(e)) => Some(e.event_type.clone()),
                            _ => None,
                        },
                        done: chunk.done,
                    };

                    if let Ok(data) = serde_json::to_string(&body) {
                        yield Ok(Event::default().data(data));
                    }

                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    let error_body = serde_json::json!({
                        "error": e.message(),
                        "done": true,
                    });
                    yield Ok(Event::default().data(error_body.to_string()));
                    break;
                }
            }
        }
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
