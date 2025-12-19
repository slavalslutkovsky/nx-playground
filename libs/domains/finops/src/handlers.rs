//! HTTP handlers for finops domain with SSE streaming

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use axum_helpers::{
    errors::responses::{
        BadRequestUuidResponse, BadRequestValidationResponse, InternalServerErrorResponse,
        NotFoundResponse,
    },
    UuidPath, ValidatedJson,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::agent::FinopsOrchestrator;
use crate::error::FinopsResult;
use crate::models::{
    ChatChunk, ChatContext, ChatMessage, ChatRequest, ChatResponse, ChatSession, CloudAccount,
    CloudResource, CreateCloudAccount, CreateSession, Recommendation, RecommendationFilter,
    ResourceFilter, SessionFilter,
};
use crate::repository::FinopsRepository;
use crate::service::FinopsService;
use serde::Deserialize;

/// Query parameters for the SSE chat stream endpoint.
/// Context is passed as a JSON string since EventSource only supports GET with query params.
#[derive(Debug, Deserialize)]
pub struct ChatStreamQuery {
    pub session_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub message: String,
    /// JSON-encoded ChatContext
    pub context: Option<String>,
}

impl ChatStreamQuery {
    /// Parse the context JSON string into ChatContext
    pub fn parse_context(&self) -> Option<ChatContext> {
        self.context
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }
}

/// Shared state for handlers
pub struct FinopsState<R: FinopsRepository + 'static> {
    pub service: Arc<FinopsService<R>>,
    pub orchestrator: Arc<FinopsOrchestrator<R>>,
}

impl<R: FinopsRepository + 'static> Clone for FinopsState<R> {
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}

impl<R: FinopsRepository + 'static> FinopsState<R> {
    pub fn new(service: FinopsService<R>, orchestrator: FinopsOrchestrator<R>) -> Self {
        Self {
            service: Arc::new(service),
            orchestrator: Arc::new(orchestrator),
        }
    }
}

/// OpenAPI documentation for FinOps API
#[derive(OpenApi)]
#[openapi(
    paths(
        // Chat endpoints
        chat_handler,
        chat_stream_handler,
        // Session endpoints
        list_sessions,
        create_session,
        get_session,
        delete_session,
        get_messages,
        // Cloud account endpoints
        list_accounts,
        connect_account,
        get_account,
        disconnect_account,
        sync_account,
        // Resource endpoints
        list_resources,
        get_resource,
        get_resource_recommendations,
        // Recommendation endpoints
        list_recommendations,
        get_recommendation,
        approve_recommendation,
        dismiss_recommendation,
    ),
    components(
        schemas(
            ChatRequest,
            ChatResponse,
            ChatMessage,
            ChatSession,
            ChatChunk,
            CloudAccount,
            CloudResource,
            Recommendation,
            CreateSession,
            CreateCloudAccount,
            SessionFilter,
            ResourceFilter,
            RecommendationFilter,
        ),
        responses(
            NotFoundResponse,
            BadRequestValidationResponse,
            BadRequestUuidResponse,
            InternalServerErrorResponse
        )
    ),
    tags(
        (name = "finops-chat", description = "FinOps AI chat endpoints"),
        (name = "finops-sessions", description = "Chat session management"),
        (name = "finops-accounts", description = "Cloud account management"),
        (name = "finops-resources", description = "Cloud resource management"),
        (name = "finops-recommendations", description = "Optimization recommendations")
    )
)]
pub struct ApiDoc;

/// Create the finops router with all HTTP endpoints
pub fn router<R: FinopsRepository + 'static>(state: FinopsState<R>) -> Router {
    Router::new()
        // Chat endpoints
        .route("/chat", post(chat_handler))
        .route("/chat/stream", get(chat_stream_handler))
        // Session management
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{id}", get(get_session).delete(delete_session))
        .route("/sessions/{id}/messages", get(get_messages))
        // Cloud accounts
        .route("/accounts", get(list_accounts).post(connect_account))
        .route("/accounts/{id}", get(get_account).delete(disconnect_account))
        .route("/accounts/{id}/sync", post(sync_account))
        // Resources
        .route("/resources", get(list_resources))
        .route("/resources/{id}", get(get_resource))
        .route(
            "/resources/{id}/recommendations",
            get(get_resource_recommendations),
        )
        // Recommendations
        .route("/recommendations", get(list_recommendations))
        .route("/recommendations/{id}", get(get_recommendation))
        .route("/recommendations/{id}/approve", post(approve_recommendation))
        .route("/recommendations/{id}/dismiss", post(dismiss_recommendation))
        .with_state(state)
}

// =============================================================================
// Chat Endpoints
// =============================================================================

/// Send a chat message and get a response
#[utoipa::path(
    post,
    path = "/chat",
    tag = "finops-chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Chat response", body = ChatResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn chat_handler<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    ValidatedJson(request): ValidatedJson<ChatRequest>,
) -> FinopsResult<Json<ChatResponse>> {
    // Get or create session
    let session = state
        .service
        .get_or_create_session(request.session_id, request.user_id, request.context)
        .await?;

    // Save user message
    state
        .service
        .save_user_message(session.id, &request.message)
        .await?;

    // Execute agent
    let response = state
        .orchestrator
        .chat(session.id, &request.message, &session.context)
        .await?;

    // Save assistant message
    state
        .service
        .save_assistant_message(
            session.id,
            &response.content,
            if response.tool_calls.is_empty() { None } else { Some(response.tool_calls.clone()) },
            None,
            None,
        )
        .await?;

    Ok(Json(response))
}

/// Stream chat response via SSE (GET for EventSource compatibility)
#[utoipa::path(
    get,
    path = "/chat/stream",
    tag = "finops-chat",
    params(
        ("session_id" = Option<Uuid>, Query, description = "Existing session ID"),
        ("user_id" = Option<Uuid>, Query, description = "User ID (optional for guests)"),
        ("message" = String, Query, description = "Chat message"),
        ("context" = Option<String>, Query, description = "JSON-encoded chat context")
    ),
    responses(
        (status = 200, description = "SSE stream of chat responses")
    )
)]
async fn chat_stream_handler<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    Query(query): Query<ChatStreamQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Parse context from JSON string
    let context = query.parse_context();

    let stream = async_stream::stream! {
        // Get or create session
        let session = match state
            .service
            .get_or_create_session(query.session_id, query.user_id, context.clone())
            .await
        {
            Ok(s) => s,
            Err(e) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(e.to_string()));
                return;
            }
        };

        // Save user message
        if let Err(e) = state.service.save_user_message(session.id, &query.message).await {
            yield Ok(Event::default()
                .event("error")
                .data(e.to_string()));
            return;
        }

        // Stream agent response
        let mut agent_stream = match state
            .orchestrator
            .chat_stream(session.id, &query.message, &session.context)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(e.to_string()));
                return;
            }
        };

        use futures::StreamExt;

        let mut full_response = String::new();
        let mut tool_calls = Vec::new();

        while let Some(chunk) = agent_stream.next().await {
            match chunk {
                Ok(ChatChunk::Text { content }) => {
                    full_response.push_str(&content);
                    yield Ok(Event::default()
                        .event("text")
                        .data(&content));
                }
                Ok(ChatChunk::ToolCall { name, arguments }) => {
                    yield Ok(Event::default()
                        .event("tool_call")
                        .data(serde_json::json!({
                            "name": name,
                            "arguments": arguments
                        }).to_string()));
                }
                Ok(ChatChunk::ToolResult { name, result }) => {
                    tool_calls.push(crate::models::ToolCallRecord {
                        name: name.clone(),
                        arguments: String::new(),
                        result: Some(result.clone()),
                        latency_ms: None,
                    });
                    yield Ok(Event::default()
                        .event("tool_result")
                        .data(serde_json::json!({
                            "name": name,
                            "result": result
                        }).to_string()));
                }
                Ok(ChatChunk::Done { session_id }) => {
                    // Save assistant message
                    let _ = state.service.save_assistant_message(
                        session_id,
                        &full_response,
                        if tool_calls.is_empty() { None } else { Some(tool_calls.clone()) },
                        None,
                        None,
                    ).await;

                    yield Ok(Event::default()
                        .event("done")
                        .data(session_id.to_string()));
                }
                Ok(ChatChunk::Error { message }) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(message));
                }
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(e.to_string()));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// =============================================================================
// Session Endpoints
// =============================================================================

/// List chat sessions
#[utoipa::path(
    get,
    path = "/sessions",
    tag = "finops-sessions",
    params(SessionFilter),
    responses(
        (status = 200, description = "List of sessions", body = Vec<ChatSession>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_sessions<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    Query(filter): Query<SessionFilter>,
) -> FinopsResult<Json<Vec<ChatSession>>> {
    let sessions = state.service.list_sessions(filter).await?;
    Ok(Json(sessions))
}

/// Create a new chat session
#[utoipa::path(
    post,
    path = "/sessions",
    tag = "finops-sessions",
    request_body = CreateSession,
    responses(
        (status = 201, description = "Session created", body = ChatSession),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn create_session<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    ValidatedJson(input): ValidatedJson<CreateSession>,
) -> FinopsResult<impl IntoResponse> {
    let session = state
        .service
        .get_or_create_session(None, input.user_id, input.context)
        .await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// Get a chat session by ID
#[utoipa::path(
    get,
    path = "/sessions/{id}",
    tag = "finops-sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session found", body = ChatSession),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_session<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<ChatSession>> {
    let session = state.service.get_session(id).await?;
    Ok(Json(session))
}

/// Delete a chat session
#[utoipa::path(
    delete,
    path = "/sessions/{id}",
    tag = "finops-sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn delete_session<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<impl IntoResponse> {
    state.service.delete_session(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get messages for a session
#[utoipa::path(
    get,
    path = "/sessions/{id}/messages",
    tag = "finops-sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<ChatMessage>),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_messages<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<Vec<ChatMessage>>> {
    let messages = state.service.get_conversation_history(id).await?;
    Ok(Json(messages))
}

// =============================================================================
// Cloud Account Endpoints
// =============================================================================

/// List cloud accounts for a user
#[utoipa::path(
    get,
    path = "/accounts",
    tag = "finops-accounts",
    params(
        ("user_id" = Uuid, Query, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of cloud accounts", body = Vec<CloudAccount>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_accounts<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    Query(params): Query<UserIdQuery>,
) -> FinopsResult<Json<Vec<CloudAccount>>> {
    let accounts = state.service.list_cloud_accounts(params.user_id).await?;
    Ok(Json(accounts))
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
struct UserIdQuery {
    user_id: Uuid,
}

/// Connect a cloud account
#[utoipa::path(
    post,
    path = "/accounts",
    tag = "finops-accounts",
    request_body = CreateCloudAccount,
    responses(
        (status = 201, description = "Account connected", body = CloudAccount),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn connect_account<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    ValidatedJson(input): ValidatedJson<CreateCloudAccount>,
) -> FinopsResult<impl IntoResponse> {
    let account = state.service.connect_cloud_account(input).await?;
    Ok((StatusCode::CREATED, Json(account)))
}

/// Get a cloud account
#[utoipa::path(
    get,
    path = "/accounts/{id}",
    tag = "finops-accounts",
    params(
        ("id" = Uuid, Path, description = "Account ID")
    ),
    responses(
        (status = 200, description = "Cloud account", body = CloudAccount),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_account<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<CloudAccount>> {
    let account = state.service.get_cloud_account(id).await?;
    Ok(Json(account))
}

/// Disconnect a cloud account
#[utoipa::path(
    delete,
    path = "/accounts/{id}",
    tag = "finops-accounts",
    params(
        ("id" = Uuid, Path, description = "Account ID")
    ),
    responses(
        (status = 204, description = "Account disconnected"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn disconnect_account<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<impl IntoResponse> {
    state.service.disconnect_cloud_account(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Trigger sync for a cloud account
#[utoipa::path(
    post,
    path = "/accounts/{id}/sync",
    tag = "finops-accounts",
    params(
        ("id" = Uuid, Path, description = "Account ID")
    ),
    responses(
        (status = 202, description = "Sync triggered"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn sync_account<R: FinopsRepository + 'static>(
    State(_state): State<FinopsState<R>>,
    UuidPath(_id): UuidPath,
) -> FinopsResult<impl IntoResponse> {
    // TODO: Trigger Redis Streams sync job
    Ok(StatusCode::ACCEPTED)
}

// =============================================================================
// Resource Endpoints
// =============================================================================

/// List resources with filters
#[utoipa::path(
    get,
    path = "/resources",
    tag = "finops-resources",
    params(ResourceFilter),
    responses(
        (status = 200, description = "List of resources", body = Vec<CloudResource>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_resources<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    Query(filter): Query<ResourceFilter>,
) -> FinopsResult<Json<Vec<CloudResource>>> {
    let resources = state.service.list_resources(filter).await?;
    Ok(Json(resources))
}

/// Get a resource by ID
#[utoipa::path(
    get,
    path = "/resources/{id}",
    tag = "finops-resources",
    params(
        ("id" = Uuid, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Resource found", body = CloudResource),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_resource<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<CloudResource>> {
    let resource = state.service.get_resource(id).await?;
    Ok(Json(resource))
}

/// Get recommendations for a resource
#[utoipa::path(
    get,
    path = "/resources/{id}/recommendations",
    tag = "finops-resources",
    params(
        ("id" = Uuid, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Resource recommendations", body = Vec<Recommendation>),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_resource_recommendations<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<Vec<Recommendation>>> {
    let recommendations = state.service.get_recommendations_for_resource(id).await?;
    Ok(Json(recommendations))
}

// =============================================================================
// Recommendation Endpoints
// =============================================================================

/// List recommendations with filters
#[utoipa::path(
    get,
    path = "/recommendations",
    tag = "finops-recommendations",
    params(RecommendationFilter),
    responses(
        (status = 200, description = "List of recommendations", body = Vec<Recommendation>),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn list_recommendations<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    Query(filter): Query<RecommendationFilter>,
) -> FinopsResult<Json<Vec<Recommendation>>> {
    let recommendations = state.service.list_recommendations(filter).await?;
    Ok(Json(recommendations))
}

/// Get a recommendation by ID
#[utoipa::path(
    get,
    path = "/recommendations/{id}",
    tag = "finops-recommendations",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    responses(
        (status = 200, description = "Recommendation found", body = Recommendation),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn get_recommendation<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<Recommendation>> {
    let recommendation = state.service.get_recommendation(id).await?;
    Ok(Json(recommendation))
}

/// Approve a recommendation
#[utoipa::path(
    post,
    path = "/recommendations/{id}/approve",
    tag = "finops-recommendations",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    responses(
        (status = 200, description = "Recommendation approved", body = Recommendation),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn approve_recommendation<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<Recommendation>> {
    let recommendation = state.service.approve_recommendation(id).await?;
    Ok(Json(recommendation))
}

/// Dismiss a recommendation
#[utoipa::path(
    post,
    path = "/recommendations/{id}/dismiss",
    tag = "finops-recommendations",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    responses(
        (status = 200, description = "Recommendation dismissed", body = Recommendation),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse)
    )
)]
async fn dismiss_recommendation<R: FinopsRepository + 'static>(
    State(state): State<FinopsState<R>>,
    UuidPath(id): UuidPath,
) -> FinopsResult<Json<Recommendation>> {
    let recommendation = state.service.dismiss_recommendation(id).await?;
    Ok(Json(recommendation))
}
