//! Stream-based handlers for tasks API.
//!
//! Fire-and-forget handlers (`/api/tasks-stream`) - Returns 202 Accepted immediately
//! after queueing the command. Best for async job processing.
//!
//! For job status, use polling pattern: check job status via `/jobs/{id}/status`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stream_worker::{Action, StreamDef, StreamProducer};
use uuid::Uuid;

use crate::error::{TaskError, TaskResult};
use crate::models::{CreateTask, TaskFilter, UpdateTask};
use crate::stream_models::{TaskCommand, TaskCommandPayload};
use crate::streams::TaskCommandStream;

// ============================================================================
// State
// ============================================================================

/// State for stream-based handlers.
#[derive(Clone)]
pub struct StreamState {
    /// Producer to send commands to the tasks:commands stream.
    pub producer: StreamProducer,
    /// Redis connection for reading results (sync mode only).
    pub redis: Arc<ConnectionManager>,
}

impl StreamState {
    /// Create a new stream state.
    pub fn new(redis: ConnectionManager) -> Self {
        let producer = StreamProducer::from_stream_def::<TaskCommandStream>(redis.clone());
        Self {
            producer,
            redis: Arc::new(redis),
        }
    }
}

// ============================================================================
// Response types for fire-and-forget
// ============================================================================

/// Response for fire-and-forget operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedResponse {
    /// The correlation ID for tracking the request.
    pub correlation_id: String,
    /// Message indicating the request was accepted.
    pub message: String,
    /// Stream the command was sent to.
    pub stream: String,
}

// ============================================================================
// Fire-and-Forget Handlers (202 Accepted)
// ============================================================================

/// List tasks via Redis stream (fire-and-forget)
///
/// Queues a list command and returns immediately.
pub async fn list_tasks_async(
    State(state): State<StreamState>,
) -> TaskResult<impl IntoResponse> {
    let filter = TaskFilter {
        project_id: None,
        status: None,
        priority: None,
        completed: None,
        limit: 50,
        offset: 0,
    };

    let command = TaskCommand::new(Action::Read, TaskCommandPayload::List(filter));
    let correlation_id = command.correlation_id.clone();

    state
        .producer
        .send(&command)
        .await
        .map_err(|e| TaskError::Stream(format!("Failed to send command: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            correlation_id,
            message: "List command queued".to_string(),
            stream: TaskCommandStream::STREAM_NAME.to_string(),
        }),
    ))
}

/// Create a task via Redis stream (fire-and-forget)
///
/// Queues a create command and returns immediately with 202 Accepted.
pub async fn create_task_async(
    State(state): State<StreamState>,
    Json(input): Json<CreateTask>,
) -> TaskResult<impl IntoResponse> {
    let command = TaskCommand::new(Action::Create, TaskCommandPayload::Create(input));
    let correlation_id = command.correlation_id.clone();

    state
        .producer
        .send(&command)
        .await
        .map_err(|e| TaskError::Stream(format!("Failed to send command: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            correlation_id,
            message: "Create command queued".to_string(),
            stream: TaskCommandStream::STREAM_NAME.to_string(),
        }),
    ))
}

/// Get a task via Redis stream (fire-and-forget)
pub async fn get_task_async(
    State(state): State<StreamState>,
    Path(id): Path<String>,
) -> TaskResult<impl IntoResponse> {
    let task_id =
        Uuid::parse_str(&id).map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    let command = TaskCommand::new(Action::Read, TaskCommandPayload::GetById { task_id });
    let correlation_id = command.correlation_id.clone();

    state
        .producer
        .send(&command)
        .await
        .map_err(|e| TaskError::Stream(format!("Failed to send command: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            correlation_id,
            message: "Get command queued".to_string(),
            stream: TaskCommandStream::STREAM_NAME.to_string(),
        }),
    ))
}

/// Update a task via Redis stream (fire-and-forget)
pub async fn update_task_async(
    State(state): State<StreamState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTask>,
) -> TaskResult<impl IntoResponse> {
    let task_id =
        Uuid::parse_str(&id).map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    let command = TaskCommand::new(
        Action::Update,
        TaskCommandPayload::Update {
            task_id,
            data: input,
        },
    );
    let correlation_id = command.correlation_id.clone();

    state
        .producer
        .send(&command)
        .await
        .map_err(|e| TaskError::Stream(format!("Failed to send command: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            correlation_id,
            message: "Update command queued".to_string(),
            stream: TaskCommandStream::STREAM_NAME.to_string(),
        }),
    ))
}

/// Delete a task via Redis stream (fire-and-forget)
pub async fn delete_task_async(
    State(state): State<StreamState>,
    Path(id): Path<String>,
) -> TaskResult<impl IntoResponse> {
    let task_id =
        Uuid::parse_str(&id).map_err(|_| TaskError::Validation("Invalid task ID".to_string()))?;

    let command = TaskCommand::new(Action::Delete, TaskCommandPayload::Delete { task_id });
    let correlation_id = command.correlation_id.clone();

    state
        .producer
        .send(&command)
        .await
        .map_err(|e| TaskError::Stream(format!("Failed to send command: {}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            correlation_id,
            message: "Delete command queued".to_string(),
            stream: TaskCommandStream::STREAM_NAME.to_string(),
        }),
    ))
}

